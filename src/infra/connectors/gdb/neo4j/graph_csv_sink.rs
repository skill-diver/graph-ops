use bb8_bolt::bb8::PooledConnection;
use bb8_bolt::bolt_client::Metadata;
use bb8_bolt::bolt_proto::message::Success;
use bb8_bolt::bolt_proto::{Message, Value};
use bb8_bolt::Manager;
use log::{info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::infra::connectors::gdb::neo4j::neo4j_database_provider::Neo4jDatabaseProvider;
use crate::infra::pi::{File, Schema, Sink, Writer};
use crate::{FeatureValueType, SeResult};

#[derive(Debug)]
pub struct Neo4jCSVSink {
    db: Arc<Neo4jDatabaseProvider>,
    schema: Schema,
}

impl Neo4jCSVSink {
    #[allow(dead_code)]
    pub(super) fn new(db: Arc<Neo4jDatabaseProvider>, schema: Schema) -> Self {
        Self { db, schema }
    }
}

pub struct Neo4jCSVWriter<'a> {
    bolt_conn: PooledConnection<'a, Manager>,
    schema: &'a Schema,
    periodic_commit: bool,
}

impl Neo4jCSVWriter<'_> {
    #[allow(dead_code)]
    pub fn set_periodic_commit(&mut self, periodic_commit: bool) {
        self.periodic_commit = periodic_commit;
    }
}

#[async_trait::async_trait(?Send)]
impl Sink<File> for Neo4jCSVSink {
    async fn create_writer(&self) -> SeResult<Box<dyn Writer<File> + '_>> {
        Ok(Box::new(Neo4jCSVWriter {
            bolt_conn: self.db.get_bolt_connection().await?,
            schema: &self.schema,
            periodic_commit: false,
        }))
    }
}

fn get_cypher_parser(x: &FeatureValueType) -> Option<&str> {
    let result = match x {
        FeatureValueType::String => return None,
        FeatureValueType::Int => "toInteger",
        FeatureValueType::Float => "toFloat",
        FeatureValueType::Boolean => "toBoolean",
        FeatureValueType::Date => "date",
        FeatureValueType::Time => "toDatetime",
        FeatureValueType::DateTime => "datetime",
        FeatureValueType::Duration => "toInteger",
        //topology and array type not supported
        _ => {
            warn!("Type not supported; Input as String.");
            return None;
        }
    };
    Some(result)
}

fn default_value(x: &FeatureValueType) -> String {
    let result = match x {
        FeatureValueType::String => "\"\"",
        FeatureValueType::Int => "0",
        FeatureValueType::Float => "0",
        FeatureValueType::Boolean => "false",
        FeatureValueType::Date => "0000-01-01",
        FeatureValueType::Time => "00:00:00",
        FeatureValueType::DateTime => "0000-01-01T00:00:00",
        FeatureValueType::Duration => "0",
        _ => "",
    };
    result.to_string()
}

fn field_to_props(
    fields_name: &[String],
    fields_type: &[FeatureValueType],
    with_header: bool,
    offset: usize,
) -> String {
    if fields_name.is_empty() {
        assert!(fields_type.is_empty());
        return "".to_string();
    }
    let mut props = HashMap::new();
    for i in 0..fields_name.iter().len() {
        let cypher_parser = get_cypher_parser(&fields_type[i]);
        let key = match with_header {
            true => format!("\"{}\"", fields_name[i]),
            false => format!("{}", i + offset),
        };
        let json_value = match cypher_parser {
            None => {
                format!("line[{}]", key)
            }
            Some(parser) => {
                format!("{}(line[{}])", parser, key)
            }
        };
        let coalesce = format!(
            "coalesce({},{})",
            json_value,
            default_value(&fields_type[i])
        );
        props.insert(fields_name[i].clone(), coalesce);
    }
    let mut s = String::from("{");
    for (k, v) in props.iter() {
        s.push_str(&format!("{}:{},", k, v).to_string());
    }
    //pop the last comma
    assert_eq!(s.pop(), Some(','));
    s.push('}');
    s
}

fn get_message(msg: &Message) -> String {
    match msg {
        Message::Success(success) => match &success.metadata().get("message") {
            Some(Value::String(s)) => s.to_string(),
            _ => "".to_string(),
        },
        Message::Failure(failure) => match &failure.metadata().get("message") {
            Some(Value::String(s)) => s.to_string(),
            _ => "".to_string(),
        },
        _ => "".to_string(),
    }
}

async fn exec_query(query: &String, conn: &mut PooledConnection<'_, Manager>) -> SeResult<()> {
    let result = conn.run(query, None, None).await.unwrap();
    let msg = get_message(&result);
    if Success::try_from(result).is_err() {
        info!("Failed: {}", &msg);
        return Err(msg.into());
    }
    let (_, result) = conn
        .pull(Some(Metadata::from_iter(vec![("n", 1)])))
        .await
        .unwrap();
    let msg = get_message(&result);
    if Success::try_from(result).is_err() {
        info!("Failed: {}", &msg);
        return Err(msg.into());
    }
    Ok(())
}

//We use merge to insert edge or nodes.
// As a result, it is recommended to have a index on primary key before merging
//When inserting edge, if either of its nodes are not already present, it is discarded.
#[async_trait::async_trait(?Send)]
impl<'a> Writer<File> for Neo4jCSVWriter<'a> {
    async fn write(&mut self, file: File) -> SeResult<()> {
        let schema = self.schema;
        //periodic commit for large csv; remove hard coding in the future
        let periodic_commit = if self.periodic_commit {
            "USING PERIODIC COMMIT 5000".to_string()
        } else {
            "".to_string()
        };
        let with_header = if file.header {
            "WITH HEADERS".to_string()
        } else {
            "".to_string()
        };
        //handle https
        let csv_load = format!(
            "{} LOAD CSV {} FROM 'file:///{}' AS line",
            periodic_commit, with_header, file.path
        );
        let csv_query = match schema {
            Schema::Tabular(s) => {
                let label = s.tlabel.as_ref().unwrap();
                let node_prop_str = field_to_props(&s.field_names, &s.field_types, file.header, 0);
                let query = format!("{} MERGE(n:{} {});", csv_load, label, node_prop_str);
                info!("Node Cypher Statement: {}", query);
                query
            }
            //Adding edges might be slow if no index have been created on the primary keys
            Schema::Edge(edge_schema) => {
                let edge_prop_str = field_to_props(
                    &edge_schema.edge_info.field_names,
                    &edge_schema.edge_info.field_types,
                    file.header,
                    2,
                );
                //assuming the keys are string
                //should refactor edge schema to add key type
                let src_key = match file.header {
                    false => "line[0]".to_string(),
                    true => format!("line[\"{}\"]", file.src_col.unwrap()),
                };
                let dst_key = match file.header {
                    false => "line[1]".to_string(),
                    true => format!("line[\"{}\"]", file.dst_col.unwrap()),
                };
                let match_node = format!(
                    "MATCH (src:{src_tlabel} {{ {src_primary_key}: {src_external_id}}}),\
                    (dst:{dst_tlabel} {{ {dst_primary_key}: {dst_external_id}}})",
                    src_tlabel = edge_schema.src_vertex_tlabel,
                    src_primary_key = edge_schema.src_vertex_primary_key,
                    src_external_id = src_key,
                    dst_tlabel = edge_schema.dst_vertex_tlabel,
                    dst_primary_key = edge_schema.dst_vertex_primary_key,
                    dst_external_id = dst_key,
                );
                let query = format!(
                    "{csv} {match_node} MERGE (src)\
                     -[e:{edge_tlabel} {prop}]-{direction}(dst);",
                    csv = csv_load,
                    match_node = match_node,
                    edge_tlabel = edge_schema.edge_info.tlabel.as_ref().unwrap(),
                    prop = edge_prop_str,
                    direction = if edge_schema.directed { ">" } else { "" },
                );
                info!("Edge Cypher Statement: {}", query);
                query
            }
        };
        exec_query(&csv_query, &mut self.bolt_conn).await
    }
}

#[cfg(test)]
mod test {
    use crate::infra::connectors::gdb::neo4j::graph_csv_sink::{exec_query, Neo4jCSVSink};
    use crate::infra::connectors::gdb::neo4j::neo4j_database_provider::Neo4jDatabaseProvider;
    use crate::infra::pi::{EdgeSchema, File, Schema, Sink, TabularSchema};
    use crate::FeatureValueType;
    use std::error::Error;
    use std::sync::Arc;
    //this test needs manual setup and might interfere with other testcases
    //hence it is ignored
    #[tokio::test]
    #[ignore]
    async fn input_csv() -> Result<(), Box<dyn Error>> {
        let db = Neo4jDatabaseProvider::new("localhost:7687", "neo4j", "ofnil", None, None)
            .await
            .unwrap();
        let arc_db = Arc::new(db);
        //it is recommended to create the index before running this test
        //since neo4j creates index in background
        let create_index =
            "CREATE INDEX idx IF NOT EXISTS FOR (n:Product) ON (n.asin);".to_string();
        exec_query(
            &create_index,
            &mut arc_db.get_bolt_connection().await.unwrap(),
        )
        .await
        .unwrap();
        let node_schema = Schema::Tabular(TabularSchema {
            field_names: vec![
                "asin".to_string(),
                "description".to_string(),
                "price".to_string(),
                "rank".to_string(),
            ],
            field_types: vec![
                FeatureValueType::String,
                FeatureValueType::String,
                FeatureValueType::String,
                FeatureValueType::String,
            ],
            tlabel: Some(String::from("Product")),
        });
        let node_sink = Neo4jCSVSink::new(arc_db.clone(), node_schema);
        let mut node_writer = node_sink.create_writer().await.unwrap();
        let node_file = File {
            path: "product.csv".to_string(),
            header: true,
            src_col: None,
            dst_col: None,
        };
        node_writer.write(node_file).await?;
        //csv has 29447 lines (header included)
        let edge_schema = EdgeSchema {
            src_vertex_tlabel: "Product".to_string(),
            dst_vertex_tlabel: "Product".to_string(),
            src_vertex_primary_key: "asin".to_string(),
            dst_vertex_primary_key: "asin".to_string(),
            directed: true,
            edge_info: TabularSchema {
                field_names: vec![],
                field_types: vec![],
                tlabel: Some("is_similar".to_string()),
            },
        };
        let edge_sink = Neo4jCSVSink::new(arc_db.clone(), Schema::Edge(edge_schema));
        let mut edge_writer = edge_sink.create_writer().await.unwrap();
        let edge_file = File {
            path: "Product_isSimilarTo_Product.csv".to_string(),
            header: true,
            src_col: Some("START_ID".to_string()),
            dst_col: Some("END_ID".to_string()),
        };
        edge_writer.write(edge_file).await?;
        //41607 lines with header
        //40067 relations created
        //Below are the clean up code
        //However, since it is non-trivial to determine if csv is corrected loaded
        //We keep the loaded files to allow manual check;
        // let delete_edge="MATCH ()-[e]->() DELETE e;".to_string();
        // let delete_node="MATCH (n) DELETE n;".to_string();
        // let drop_index="DROP INDEX idx;".to_string();
        // exec_query(&delete_edge, &mut arc_db.get_bolt_connection().await.unwrap()).await.unwrap();
        // exec_query(&delete_node, &mut arc_db.get_bolt_connection().await.unwrap()).await.unwrap();
        // exec_query(&drop_index, &mut arc_db.get_bolt_connection().await.unwrap()).await.unwrap();
        Ok(())
    }
}
