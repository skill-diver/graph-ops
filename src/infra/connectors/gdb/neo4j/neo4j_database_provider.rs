mod neo4j_query_parser;
pub use neo4j_query_parser::Neo4jQueryParser;
mod plan_op;
mod plan_op_constant;

use super::PULL_SIZE;
use crate::{
    feature::EdgeEntity,
    feature::{ResourceOp, VertexEntity},
    fields,
    infra::pi::SchemaProvider,
    Entity, FeatureRegistry, FeatureValueType, Field, Graph, InfraIdentifier, Variant,
};
use bb8_bolt::{
    bb8::{ManageConnection, Pool, PooledConnection},
    bolt_client::error::{CommunicationError, ConnectionError, Error as ClientError},
    bolt_client::Metadata,
    bolt_proto,
    bolt_proto::error::Error as ProtocolError,
    bolt_proto::message::Record,
    bolt_proto::{version::*, Value},
    Manager,
};
use log::{debug, error, info};
use std::io::ErrorKind::{ConnectionAborted, ConnectionRefused};
use std::sync::{Arc, Mutex, MutexGuard};
use std::{collections::HashMap, error::Error};
use tokio::time::{sleep, Duration};

/// Supporting neo4j community edition, which supports a single database in each neo4j instance.
#[derive(Clone, Debug)]
pub struct Neo4jDatabaseProvider {
    bolt_conn_pool: Pool<Manager>,
    node_field_resource: Arc<Mutex<Vec<Record>>>,
    rel_field_resource: Arc<Mutex<Vec<Record>>>,
    neo4j_infra_id: Option<InfraIdentifier>,
}

impl Neo4jDatabaseProvider {
    pub async fn new(
        bolt_uri: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        max_pool_size: Option<u32>, // default: 128
        infra_id: Option<InfraIdentifier>,
    ) -> Result<Self, Box<dyn Error>> {
        let manager = Manager::new(
            bolt_uri.into(),
            None::<String>,
            [V4_4, 0, 0, 0],
            Metadata::from_iter(vec![
                ("user_agent", "ofnil-bolt/1.0"),
                ("scheme", "basic"),
                ("principal", username.into().as_str()),
                ("credentials", password.into().as_str()),
            ]),
        )
        .await?;

        let mut count_retry: u32 = 0;
        loop {
            match manager.connect().await {
                Err(ClientError::ConnectionError(connection_err)) => {
                    count_retry += 1;
                    match connection_err {
                        ConnectionError::HandshakeFailed(_) => {
                            error!(
                                "bolt connection manager handshake failed: {}",
                                connection_err
                            );
                        }
                        ConnectionError::IoError(io_error) => match io_error.kind() {
                            ConnectionRefused => {
                                error!(
                                        "bolt connection manager connection io error, neo4j server is not started or the given port is wrong: {:?}",
                                        io_error
                                    );
                                return Err(Box::new(ConnectionError::IoError(io_error)));
                            }
                            _ => {
                                error!(
                                    "bolt connection manager connection io error {:?}",
                                    io_error
                                );
                                return Err(Box::new(ConnectionError::IoError(io_error)));
                            }
                        },
                    }
                }
                Err(ClientError::CommunicationError(communication_err)) => {
                    match communication_err.as_ref() {
                        CommunicationError::InvalidResponse { .. } => {
                            error!(
                                "bolt connection manager invalid response : {:?}",
                                communication_err
                            );
                        }
                        CommunicationError::InvalidState { .. } => {
                            error!(
                                "bolt connection manager invalid state: {:?}",
                                communication_err
                            );
                        }
                        CommunicationError::UnsupportedOperation { .. }
                        | CommunicationError::ProtocolError { .. } => {
                            panic!(
                                "bolt connection manager communication error: {:?}",
                                communication_err
                            );
                        }
                        CommunicationError::IoError(io_error) => {
                            match io_error.kind() {
                                ConnectionAborted => {
                                    let inner_err = io_error.get_ref().unwrap_or_else(|| {
                                        panic!(
                                            "bolt connection manager connection aborted : {:?}",
                                            io_error
                                        )
                                    });
                                    let err_str = inner_err.to_string();
                                    let server_code = err_str
                                        .split("\"code\": String(\"")
                                        .nth(1)
                                        .unwrap()
                                        .split('\"')
                                        .next()
                                        .unwrap();
                                    match server_code {
                                        // TODO(Pond): support other error messages from server
                                        // https://neo4j.com/docs/status-codes/current/errors/all-errors/
                                        "Neo.ClientError.Security.Unauthorized" => panic!(
                                            "bolt connection manager communication aborted, wrong username or password : {:?}", 
                                            io_error),
                                        _ => panic!(
                                            "bolt connection manager connection aborted with code : {}",
                                            server_code
                                        )
                                    }
                                }
                                _ => {
                                    panic!(
                                        "bolt connection manager communication io error : {:?}",
                                        io_error
                                    );
                                }
                            }
                        }
                    }
                }
                Err(ClientError::ProtocolError(protocol_err)) => match protocol_err {
                    ProtocolError::ConversionError(conversion_err) => {
                        panic!(
                            "bolt connection manager conversion error : {}",
                            conversion_err
                        );
                    }
                    ProtocolError::SerializationError(serialization_err) => {
                        panic!(
                            "bolt connection manager serialization error : {}",
                            serialization_err
                        );
                    }
                    ProtocolError::DeserializationError(deserialization_err) => {
                        panic!(
                            "bolt connection manager deserialization error : {}",
                            deserialization_err
                        );
                    }
                },
                Ok(_) => {
                    info!("bolt connection manager handshake succeeded");
                    break;
                }
            }

            if count_retry == 3 {
                panic!("bolt connection manager failed");
            }
            info!("retrying bolt connection manager");
            sleep(Duration::from_secs(2_u64.pow(count_retry))).await;
        }

        // Create a connection pool. This should be shared across your application.
        let pool = Pool::builder()
            .max_size(max_pool_size.unwrap_or(128))
            .build(manager)
            .await?;

        Ok(Self {
            bolt_conn_pool: pool,
            node_field_resource: Arc::new(Mutex::new(Vec::new())),
            rel_field_resource: Arc::new(Mutex::new(Vec::new())),
            neo4j_infra_id: infra_id,
        })
    }

    // A parser borrows a mutable reference of the connection to ensure exclusive usage for `execute` and `pull`
    pub async fn parse_query(&self, query: &str) -> Result<Neo4jQueryParser<'_>, Box<dyn Error>> {
        let bolt_conn = self.get_bolt_connection().await?;
        Neo4jQueryParser::new(query, bolt_conn).await
    }

    pub async fn get_bolt_connection(
        &self,
    ) -> Result<PooledConnection<Manager>, Box<Neo4jDatabaseProviderError>> {
        self.bolt_conn_pool.get().await.map_err(|e| {
            Box::new(Neo4jDatabaseProviderError::BoltConnection(format!(
                "get bolt connection failed: {e}"
            )))
        })
    }

    // requires neo4j.gds to be installed
    pub async fn check_named_graph_exists(&self, graph_name: &str) -> Result<bool, Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let check_graph_exists_query =
            format!("CALL gds.graph.exists('{graph_name}') YIELD graphName, exists");
        info!("check_graph_exists_query: {}", check_graph_exists_query);
        let msg = bolt_conn
            .run(check_graph_exists_query, None, None)
            .await
            .unwrap();
        info!("check graph exists msg: {:?}", msg);
        let (records, msg) = bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        info!(
            "check named graph exisits: {:?}, message: {:?}",
            records, msg
        );

        if let Value::Boolean(named_graph_exists) =
            records.first().unwrap().fields().get(1).unwrap().to_owned()
        {
            Ok(named_graph_exists)
        } else {
            Err(Box::new(Neo4jDatabaseProviderError::BoltMessageError(
                "check named graph exists failed",
            )))
        }
    }

    pub async fn drop_named_graph(&self, graph_name: &str) -> Result<(), Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let drop_graph_query = format!("CALL gds.graph.drop('{graph_name}')");
        info!("drop_graph_query: {}", drop_graph_query);
        let msg = bolt_conn.run(drop_graph_query, None, None).await.unwrap();
        info!("drop graph query result: {:?}", msg);

        let (records, msg) = bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        info!("drop named graph: {:?}, message: {:?}", records, msg);

        Ok(())
    }

    pub async fn get_id_mapping(
        &self,
        node_label: &str,
        primary_key: &str,
    ) -> Result<HashMap<i64, String>, Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let mut id_mapping = HashMap::<i64, String>::new();
        let id_mapping_query =
            format!("MATCH (n:{node_label}) RETURN id(n) AS id, n.{primary_key} AS external_id");
        info!("id_mapping_query: {}", id_mapping_query);
        let msg = bolt_conn.run(id_mapping_query, None, None).await.unwrap();
        info!("id mapping query result: {:?}", msg);
        info!("constructing id mapping...");
        loop {
            let (records, msg) = bolt_conn
                .pull(Some(Metadata::from_iter(vec![("n", PULL_SIZE)])))
                .await
                .unwrap();
            let success = bolt_proto::message::Success::try_from(msg).unwrap();
            for record in records {
                // info!("id mapping record: {:?}", record);
                if let Value::Integer(id) = record.fields().get(0).unwrap() {
                    if let Value::String(primary_key) = record.fields().get(1).unwrap() {
                        debug!("id: {}, reviewer_id: {}", id, primary_key);
                        id_mapping.insert(*id, primary_key.to_owned());
                    }
                }
            }

            if !(success.metadata().contains_key("has_more")
                && success.metadata()["has_more"] == Value::Boolean(true))
            {
                break;
            }
        }
        info!("id_mapping size: {:?}", id_mapping.len());

        Ok(id_mapping)
    }
}

#[async_trait::async_trait]
impl SchemaProvider for Neo4jDatabaseProvider {
    async fn register_graph(&self, registry: &FeatureRegistry) -> Result<Graph, Box<dyn Error>> {
        let entities = self.get_all_entities().await?;

        registry
            .register_resources(&entities.iter().collect())
            .await?;
        let mut fields = Vec::new();
        self.get_node_field_resource().await?;
        self.get_rel_field_resource().await?;
        for entity in &entities {
            fields.push(self.get_fields(entity).await?);
        }
        let entities = entities
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| {
                if let Entity::Vertex(v) = entity {
                    Entity::Vertex(VertexEntity {
                        primary_key: fields[idx].first().unwrap().name.to_string(),
                        ..v
                    })
                } else {
                    entity
                }
            })
            .collect::<Vec<_>>();
        let fields = fields.iter().flatten().collect();
        registry.register_resources(&fields).await?;
        let graph = Graph::new(
            // TODO(tatiana): use the database name? or the infra id name?
            "neo4j",
            Variant::Default(),
            entities.iter().collect(),
            self.get_infra_id(),
        );
        registry.register_resource(&graph).await?;
        Ok(graph)
    }

    async fn get_node_field_resource(&self) -> Result<(), Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let get_rel_fields_query = "CALL db.schema.nodeTypeProperties()".to_string();
        info!("node_type_properites_query: {}", get_rel_fields_query);
        let _msg = bolt_conn
            .run(get_rel_fields_query, None, None)
            .await
            .unwrap();
        let mut record_collector = Vec::new();
        info!("constructing get fields...");
        loop {
            let (records, msg) = bolt_conn
                .pull(Some(Metadata::from_iter(vec![("n", PULL_SIZE)])))
                .await
                .unwrap();
            let success = bolt_proto::message::Success::try_from(msg).unwrap();
            for record in records {
                record_collector.push(record);
            }
            if !(success.metadata().contains_key("has_more")
                && success.metadata()["has_more"] == Value::Boolean(true))
            {
                break;
            }
        }
        // Drop the `bolt_conn` connection before modifying `self.node_field_resource`
        drop(bolt_conn);
        for record in record_collector {
            self.node_field_resource.lock().unwrap().push(record);
        }
        Ok(())
    }

    async fn get_rel_field_resource(&self) -> Result<(), Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let get_rel_fields_query = "CALL db.schema.relTypeProperties()".to_string();
        info!("node_type_properites_query: {}", get_rel_fields_query);
        let _msg = bolt_conn
            .run(get_rel_fields_query, None, None)
            .await
            .unwrap();
        let mut record_collector = Vec::new();
        info!("constructing get fields...");
        loop {
            let (records, msg) = bolt_conn
                .pull(Some(Metadata::from_iter(vec![("n", PULL_SIZE)])))
                .await
                .unwrap();
            let success = bolt_proto::message::Success::try_from(msg).unwrap();
            for record in records {
                record_collector.push(record);
            }
            if !(success.metadata().contains_key("has_more")
                && success.metadata()["has_more"] == Value::Boolean(true))
            {
                break;
            }
        }
        // Drop the `bolt_conn` connection before modifying `self.rel_field_resource`
        drop(bolt_conn);
        for record in record_collector {
            self.rel_field_resource.lock().unwrap().push(record);
        }
        Ok(())
    }

    async fn get_all_entities(&self) -> Result<Vec<Entity>, Box<dyn Error>> {
        let mut bolt_conn = self.get_bolt_connection().await?;

        let mut all_entities = Vec::new();
        let all_entities_query = "CALL db.schema.visualization".to_string();
        info!("all_entities_query: {}", all_entities_query);
        let msg = bolt_conn.run(all_entities_query, None, None).await.unwrap();
        info!("all entities query result: {:?}", msg);
        info!("constructing all entities...");
        let mut vertex_id2vertex = HashMap::<String, Entity>::new();
        loop {
            let (records, msg) = bolt_conn
                .pull(Some(Metadata::from_iter(vec![("n", PULL_SIZE)])))
                .await
                .unwrap();
            let success = bolt_proto::message::Success::try_from(msg).unwrap();
            // TODO(Runlong): Entity name
            for record in records {
                // according to visualization defination, node will be stored in the first position
                // and relationship will be stored in second position. Store them saperately
                if let Value::List(node_entity_list) = record.fields().get(0).unwrap() {
                    for node_entity in node_entity_list {
                        if let Value::Node(node_entity) = &node_entity {
                            let temp_entity = Entity::Vertex(VertexEntity {
                                name: match node_entity.properties().get("name").unwrap() {
                                    Value::String(_name) => Ok(_name.clone()),
                                    _ => Err("node name is not string"),
                                }
                                .unwrap(),
                                tlabel: ((node_entity.labels()[0]).to_owned()),
                                primary_key: "id".to_string(), // a dummy primary key, to be specified according to the properties
                                variant: Variant::Default(),
                            });
                            vertex_id2vertex.insert(
                                (node_entity.node_identity()).to_string(),
                                temp_entity.clone(),
                            );
                            all_entities.push(temp_entity);
                        }
                    }
                }
                if let Value::List(edge_entity_list) = record.fields().get(1).unwrap() {
                    for edge_entity in edge_entity_list {
                        if let Value::Relationship(edge_entity) = &edge_entity {
                            let _src_vertex_entity = vertex_id2vertex
                                .get(&edge_entity.start_node_identity().to_string())
                                .unwrap();
                            let _dst_vertex_entity = vertex_id2vertex
                                .get(&edge_entity.end_node_identity().to_string())
                                .unwrap();
                            let temp_entity = Entity::Edge(EdgeEntity {
                                name: edge_entity.rel_type().to_string(),
                                variant: Variant::Default(),
                                tlabel: edge_entity.rel_type().to_string(),
                                src_tlabel: _src_vertex_entity.tlabel().to_string(),
                                dst_tlabel: _dst_vertex_entity.tlabel().to_string(),
                                src_entity_id: _src_vertex_entity.resource_id(),
                                dst_entity_id: _dst_vertex_entity.resource_id(),
                                directed: false,
                                primary_key: None,
                            });
                            all_entities.push(temp_entity);
                        }
                    }
                }
            }
            if !(success.metadata().contains_key("has_more")
                && success.metadata()["has_more"] == Value::Boolean(true))
            {
                break;
            }
        }
        info!("all entities size: {:?}", all_entities.len());

        Ok(all_entities)
    }

    async fn get_fields(&self, _entity: &Entity) -> Result<Vec<Field>, Box<dyn Error>> {
        let mut record_collector = Vec::new();
        let mut fields = Vec::new();
        let field_source = match _entity {
            Entity::Vertex(_entity) => &self.node_field_resource,
            Entity::Edge(_entity) => &self.rel_field_resource,
        };
        let mut guard: MutexGuard<'_, Vec<Record>> = field_source.lock().unwrap();
        let inner: &mut Vec<Record> = &mut guard;
        for record in inner {
            if let Value::String(id) = record.fields().get(0).unwrap() {
                match _entity {
                    Entity::Vertex(_entity) => {
                        if let Value::String(label_name) = record.fields().get(2).unwrap() {
                            if let Value::List(label_type) = record.fields().get(3).unwrap() {
                                if let Value::String(label_type_item) = label_type.get(0).unwrap() {
                                    // hard code part for pairing tlabel
                                    let mut format_helper = String::from(":`");
                                    format_helper.push_str((_entity.tlabel).as_str());
                                    format_helper.push('`');

                                    if format_helper == *id {
                                        let store_type = match label_type_item.as_ref() {
                                            "String" => FeatureValueType::String,
                                            // TODO(Pond): add Float value type
                                            _ => FeatureValueType::Int,
                                        };
                                        record_collector.push((label_name.to_owned(), store_type));
                                    }
                                }
                            }
                        }
                    }
                    Entity::Edge(_entity) => {
                        if let Value::String(label_name) = record.fields().get(1).unwrap() {
                            if let Value::List(label_type) = record.fields().get(2).unwrap() {
                                if let Value::String(label_type_item) = label_type.get(0).unwrap() {
                                    // hard code part for pairing tlabel
                                    let mut format_helper = String::from(":`");
                                    format_helper.push_str((_entity.tlabel).as_str());
                                    format_helper.push('`');
                                    if format_helper == *id {
                                        let store_type = match label_type_item.as_ref() {
                                            "String" => FeatureValueType::String,
                                            // TODO(Pond): add Float value type
                                            _ => FeatureValueType::Int,
                                        };
                                        record_collector.push((label_name.to_owned(), store_type));
                                    }
                                }
                            }
                        }
                    }
                };
            }
            let mut name_type_tuples = Vec::new();
            name_type_tuples.reserve(record_collector.len());
            for (name, vtype) in &record_collector {
                name_type_tuples.push((
                    name.as_str(),
                    vtype.to_owned(), // FeatureValueType::from_str(vtype).map_err(|e| PyValueError::new_err(e.to_string()))?,
                ));
            }

            fields = fields! {
                name_type_tuples,
                &_entity,
                Variant::Default(),
                self.neo4j_infra_id.clone(),
            };
        }

        info!("get fields size: {:?}", fields.len());

        Ok(fields)
    }

    fn get_infra_id(&self) -> Option<InfraIdentifier> {
        self.neo4j_infra_id.clone()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Neo4jDatabaseProviderError {
    #[error("Bolt message does not match provider implementation. {0}")]
    BoltMessageError(&'static str),
    #[error("Error parsing plan in EXPLAIN result. {0}")]
    PlanParseError(&'static str),
    // TODO(han): refactor: clippy thinks all variants with `*Error` suffix are redundant, refactor to make things consistent later
    #[error("Error getting bolt connection from pool. {0}")]
    BoltConnection(String),
    #[error("Error projecting graph. Query: {query}. Error message: {error_msg}")]
    GraphProjection { query: String, error_msg: String },
}

#[tokio::test]
async fn test_invalid_query() -> Result<(), Box<dyn Error>> {
    use super::QueryParser;
    use log::debug;
    let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", Some(1), None).await?;
    let parser1 = db
        .parse_query("MATCH (n:Reviewer)-(m) RETURN n.reviewerID")
        .await?;
    let q1 = parser1.validate_query(None, 2);
    assert!(q1.is_err(), "query should be invalid due to syntax error");
    debug!("{}", q1.unwrap_err());

    Ok(())
}

#[tokio::test]
async fn test_get_all_entities() -> Result<(), Box<dyn Error>> {
    let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", Some(1), None).await?;
    let _sp = db.get_all_entities().await?;

    Ok(())
}

#[tokio::test]
async fn test_get_fields() -> Result<(), Box<dyn Error>> {
    let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", Some(1), None).await?;
    let _sp = db.get_all_entities().await?;
    db.get_node_field_resource().await?;
    db.get_rel_field_resource().await?;
    let mut _field = Vec::new();
    for item in _sp {
        _field.push(db.get_fields(&item).await?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use log::info;
    use std::{collections::HashMap, error::Error};

    use super::Neo4jDatabaseProvider;
    use crate::{infra::pi::QueryParser, transformation::InputSchema, FeatureValueType};

    fn populate_test_input_schema() -> InputSchema {
        let mut res = InputSchema {
            vertex_entities: HashMap::new(),
            edge_entities: HashMap::new(),
        };
        let mut category_fields = HashMap::new();
        category_fields.insert("name".to_string(), FeatureValueType::String);
        res.vertex_entities.insert(
            "Category".to_string(),
            ("default/Entity/Category".to_string(), category_fields),
        );
        let mut review_fields = HashMap::new();
        review_fields.insert("overall".to_string(), FeatureValueType::Float);
        res.vertex_entities.insert(
            "Review".to_string(),
            ("default/Entity/Review".to_string(), review_fields),
        );
        res.vertex_entities.insert(
            "Product".to_string(),
            ("default/Entity/Product".to_string(), HashMap::new()),
        );
        let mut reviewer_fields = HashMap::new();
        reviewer_fields.insert("name".to_string(), FeatureValueType::String);
        res.vertex_entities.insert(
            "Reviewer".to_string(),
            ("default/Entity/Reviewer".to_string(), reviewer_fields),
        );
        res.edge_entities.insert(
            "belongsTo".to_string(),
            (
                "default/Entity/belongsTo/Product/Category".to_string(),
                HashMap::new(),
            ),
        );
        res.edge_entities.insert(
            "rates".to_string(),
            (
                "default/Entity/rates/Review/Product".to_string(),
                HashMap::new(),
            ),
        );
        res.edge_entities.insert(
            "isWrittenBy".to_string(),
            (
                "default/Entity/isWrittenBy/Review/Reviewer".to_string(),
                HashMap::new(),
            ),
        );
        res
    }

    // TODO(han): fix the test, remove the hardcoded neo4j uri
    #[tokio::test]
    async fn test_query1() -> Result<(), Box<dyn Error>> {
        let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", None, None).await?;
        let parser = db.parse_query(
         "MATCH (p: Product)-[:belongsTo]->(cat: Category {name: \" Books\"}) MATCH (r1: Review)-[:rates]->(p)<-[:rates]-(r2: Review) MATCH (r1: Review)-[:isWrittenBy]->(u1: Reviewer) MATCH (r2: Review)-[:isWrittenBy]->(u2: Reviewer) RETURN u1, u2",
     ).await?;

        let schema = populate_test_input_schema();
        parser.validate_query(Some(&schema), 2)?;
        let output = parser.get_output_graph_schema(&schema)?;
        assert!(output.src.tlabel == Some("Reviewer".to_owned()));
        assert!(output.dst.tlabel == Some("Reviewer".to_owned()));
        Ok(())
    }

    #[tokio::test]
    async fn test_query2() -> Result<(), Box<dyn Error>> {
        let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", None, None).await?;
        let parser = db
            .parse_query(
                "MATCH (u: Reviewer)<-[:isWrittenBy]-(: Review)-[:rates]->(p: Product) RETURN u, p",
            )
            .await?;
        let schema = populate_test_input_schema();
        parser.validate_query(Some(&schema), 2)?;
        let output = parser.get_output_graph_schema(&schema)?;
        assert!(output.src.tlabel == Some("Reviewer".to_owned()));
        assert!(output.dst.tlabel == Some("Product".to_owned()));
        Ok(())
    }

    #[tokio::test]
    async fn test_query3() -> Result<(), Box<dyn Error>> {
        let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", None, None).await?;
        let parser = db
        .parse_query(
            "MATCH (u1: Reviewer)<-[:isWrittenBy]-(r1: Review)-[:rates]->(:Product)<-[:rates]-(r2: Review)-[:isWrittenBy]->(u2: Reviewer) WHERE r1.overall=r2.overall RETURN u1 as src, u2 as dst"
        )
        .await?;
        let schema = populate_test_input_schema();
        parser.validate_query(Some(&schema), 2)?;
        let output = parser.get_output_graph_schema(&schema)?;
        assert!(output.src.tlabel == Some("Reviewer".to_owned()));
        assert!(output.dst.tlabel == Some("Reviewer".to_owned()));
        Ok(())
    }

    #[tokio::test]
    async fn test_query4() -> Result<(), Box<dyn Error>> {
        let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", None, None).await?;
        let parser = db
            .parse_query(
                "MATCH (u:Reviewer)<-[:isWrittenBy]-(r:Review) RETURN ID(u) as u, r.overall",
            )
            .await?;
        let schema = populate_test_input_schema();
        parser.validate_query(Some(&schema), 2)?;
        let output = parser.get_output_graph_schema(&schema)?;
        assert!(output.src.tlabel == Some("Reviewer".to_owned()));
        assert!(output.dst.tlabel.is_none());
        info!("{:#?}", output.edge);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_conn() -> Result<(), Box<dyn Error>> {
        let db =
            Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", Some(4), None).await?;

        let mut conn = db.get_bolt_connection().await?;
        conn.run("MATCH (n) RETURN n", None, None).await?;
        let mut conn = db.get_bolt_connection().await?;
        conn.run("MATCH (n) RETURN n", None, None).await?;
        let mut conn = db.get_bolt_connection().await?;
        conn.run("MATCH (n) RETURN n", None, None).await?;
        let mut conn = db.get_bolt_connection().await?;
        conn.run("MATCH (n) RETURN n", None, None).await?;
        match db.get_bolt_connection().await {
            Ok(_) => panic!("Should not be able to get more connections"),
            Err(e) => info!("Cannot get more connections, {}", e),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_query() -> Result<(), Box<dyn Error>> {
        let db =
            Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", Some(1), None).await?;
        let parser1 = db
            .parse_query("MATCH (n:Reviewer)-(m) RETURN n.reviewerID")
            .await?;
        let q1 = parser1.validate_query(None, 2);
        assert!(q1.is_err(), "query should be invalid due to syntax error");
        info!("{}", q1.unwrap_err());

        Ok(())
    }
}
