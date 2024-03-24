use super::{
    transformation_plan::{BuiltInOp, TransformationOp},
    DataFrame, DataIdT, DataTransformationContext, GraphBase, InputSchema, TransformationArgs,
    TransformationData,
};
use crate::{
    feature::{ResourceId, ResourceOp},
    infra::{
        connectors::Neo4jConnector,
        pi::{EdgeSchema, QueryParser, Schema, TabularSchema, GAF},
    },
    Field, InfraIdentifier, Topology, TopologyType,
};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, rc::Rc};

pub trait CypherTransformation {
    fn cypher_to_graph(
        &self,
        query: &str,
        gdb_provider: &Neo4jConnector,
    ) -> Result<Rc<dyn GraphBase>, Box<dyn Error>>;
    fn cypher_to_dataframe(
        &self,
        query: &str,
        gdb_provider: &Neo4jConnector,
    ) -> Result<Rc<DataFrame>, Box<dyn Error>>;
    fn get_input_schema(&self) -> InputSchema;
    fn get_fv(&self, tlabel: Option<String>, entity_type: &str) -> Option<(String, Vec<Field>)>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CypherResultGraph {
    pub(super) context: DataTransformationContext,
    pub(super) graph: DataIdT, // graph data id
    pub(super) query: String,

    // vertex_fvs include the fvs of the src and dst node.
    pub vertex_fvs: HashMap<String, (String, Vec<Field>)>, // tlabel, (view name, fields)
    // only one edge, we use HashMap to keep the same interface as GraphBase Trait.
    pub edge_fvs: HashMap<String, (String, Vec<Field>)>, // tlabel, (view name, fields)
    // vertex_entities include the entities of the src and dst node.
    pub vertex_entities: HashMap<String, ResourceId>, // tlabel, entity id
    // only one edge, we use HashMap to keep the same interface as GraphBase Trait.
    pub edge_entities: HashMap<String, ResourceId>, // tlabel, entity id
    pub topology_type: Option<TopologyType>,
    // add role_to_entity to get the src and dst resource id easily.
    pub role_to_entity: HashMap<String, Option<ResourceId>>, // role is choosen from {"src", "dst", "edge"}
}

impl<T> CypherTransformation for T
where
    T: GraphBase,
{
    fn cypher_to_graph(
        &self,
        query: &str,
        gdb_provider: &Neo4jConnector,
    ) -> Result<Rc<dyn GraphBase>, Box<dyn Error>> {
        let db = gdb_provider.get_database();
        let parser = block_on(db.parse_query(query))?;
        let input_schema = self.get_input_schema();
        let out_schema = match parser.validate_query(Some(&input_schema), 2) {
            Ok(_) => parser.get_output_graph_schema(&input_schema)?,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        let (src_schema_entity, dst_schema_entity) = out_schema.get_node_schema();
        let edge_schema_entity = out_schema.get_edge_schema();
        // get tlabel and ResourceID
        let (src_tlabel, src_entity) = src_schema_entity.get_tlabel_id();
        let (dst_tlabel, dst_entity) = dst_schema_entity.get_tlabel_id();
        let (edge_tlabel, edge_entity) = edge_schema_entity.get_tlabel_id();

        // get fv
        let src_fv = self.get_fv(src_tlabel.to_owned(), "node");
        let dst_fv = self.get_fv(dst_tlabel.to_owned(), "node");
        let edge_fv = self.get_fv(edge_tlabel.to_owned(), "edge");

        let mut vertex_fvs = HashMap::new();
        let mut edge_fvs = HashMap::new();
        let mut vertex_entities = HashMap::new();
        let mut edge_entities = HashMap::new();
        let mut role_to_entity = HashMap::new();
        // FIXME(tatiana): the fields should be obtained from GraphSchemaEntity instead of directly from self
        vertex_fvs.insert(
            src_tlabel.to_owned().unwrap_or_default(),
            src_fv.unwrap_or_default(),
        );
        vertex_fvs.insert(
            dst_tlabel.to_owned().unwrap_or_default(),
            dst_fv.unwrap_or_default(),
        );
        edge_fvs.insert(
            edge_tlabel.to_owned().unwrap_or_default(),
            edge_fv.unwrap_or_default(),
        );
        vertex_entities.insert(
            src_tlabel.to_owned().unwrap_or_default(),
            src_entity.to_owned().unwrap_or_default(),
        );
        vertex_entities.insert(
            dst_tlabel.to_owned().unwrap_or_default(),
            dst_entity.to_owned().unwrap_or_default(),
        );
        edge_entities.insert(
            edge_tlabel.to_owned().unwrap_or_default(),
            edge_entity.to_owned().unwrap_or_default(),
        );
        role_to_entity.insert("src".to_owned(), src_entity.to_owned());
        role_to_entity.insert("dst".to_owned(), dst_entity.to_owned());
        role_to_entity.insert("edge".to_owned(), edge_entity.to_owned());

        let res = Rc::new(CypherResultGraph {
            context: self.get_context().new_data_context(None),
            graph: self.get_data_id(),
            query: query.to_string(),
            vertex_fvs,
            edge_fvs,
            vertex_entities,
            edge_entities,
            topology_type: None,
            role_to_entity,
        });

        self.get_context().register_data(&res);

        Ok(res)
    }

    #[allow(unused)]
    fn cypher_to_dataframe(
        &self,
        query: &str,
        gdb_provider: &Neo4jConnector,
    ) -> Result<Rc<DataFrame>, Box<dyn Error>> {
        let new_data_context = self.get_context().new_data_context(None);
        let name = format!("cypher_to_dataframe{}", new_data_context.id);
        todo!()
    }

    fn get_input_schema(&self) -> InputSchema {
        let vertex_fvs = self.get_vertex_fvs();
        let v_entities = self.get_vertex_entities();
        let edge_fvs = self.get_edge_fvs();
        let e_entities = self.get_edge_entities();
        InputSchema::from(vertex_fvs, edge_fvs, v_entities, e_entities)
    }

    fn get_fv(&self, tlabel: Option<String>, entity_type: &str) -> Option<(String, Vec<Field>)> {
        let fv = match entity_type {
            "vertex" => self
                .get_vertex_fvs()
                .get(&tlabel.unwrap_or_default())
                .cloned(),
            "edge" => self
                .get_edge_fvs()
                .get(&tlabel.unwrap_or_default())
                .cloned(),
            _ => None,
        };
        fv
    }
}

#[typetag::serde]
impl TransformationData for CypherResultGraph {
    fn get_context(&self) -> &DataTransformationContext {
        &self.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        let mut iter = self.vertex_entities.iter();
        Box::new(BuiltInOp::new(
            GAF::Cypher,
            TransformationArgs::new_cypher_args(
                self.query.to_string(),
                Schema::Edge(EdgeSchema {
                    src_vertex_tlabel: iter.next().unwrap().0.clone(),
                    dst_vertex_tlabel: iter.next().unwrap().0.clone(),
                    // TODO(tatiana): fill primary keys according to query parsing result
                    src_vertex_primary_key: "id".to_string(),
                    dst_vertex_primary_key: "id".to_string(),
                    directed: true,
                    edge_info: TabularSchema {
                        tlabel: self
                            .edge_entities
                            .iter()
                            .next()
                            .map(|(tlabel, _)| tlabel.clone()),
                        // TODO(tatiana): fill schema according to query parsing result
                        field_names: Vec::new(),
                        field_types: Vec::new(),
                    },
                }),
            ),
            self.get_context().get_transformation_args().clone(),
        ))
    }

    fn get_func(&self) -> GAF {
        GAF::Cypher
    }
}

impl GraphBase for CypherResultGraph {
    fn get_vertex_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        &self.vertex_fvs
    }

    fn get_edge_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        &self.edge_fvs
    }

    fn get_vertex_entities(&self) -> &HashMap<String, ResourceId> {
        &self.vertex_entities
    }

    fn get_edge_entities(&self) -> &HashMap<String, ResourceId> {
        &self.edge_entities
    }

    fn get_topology_type(&self) -> &Option<TopologyType> {
        &self.topology_type
    }

    fn vertices(&self) -> Rc<dyn GraphBase> {
        todo!()
    }

    fn edges(&self) -> Rc<dyn GraphBase> {
        todo!()
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn vertices_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        todo!()
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn edges_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        todo!()
    }

    fn export_topology(&self, name: &str, sink_infra_id: &InfraIdentifier) -> Topology {
        let res = Topology {
            name: name.to_string(),
            transformation_id: Some(self.get_context().get_transformation_id()),
            topology_type: self.topology_type.clone(),
            sink_infra_id: Some(sink_infra_id.clone()),
            edge_entity_id: self.role_to_entity["edge"].to_owned(),
            src_node_entity_id: self.role_to_entity["src"].to_owned(),
            dst_node_entity_id: self.role_to_entity["dst"].to_owned(),
            ..Default::default()
        };
        self.get_context()
            .export_resource(self.get_data_id(), res.resource_id(), sink_infra_id);
        res
    }

    fn export_df(&self, _sink_infra_id: &InfraIdentifier) -> Vec<Field> {
        // TODO(kaili): vertex fields
        self.edge_fvs
            .values()
            .flat_map(|(_, fields)| fields.clone())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CypherResultDataFrame {
    pub query: String,
    context: DataTransformationContext,
}

#[typetag::serde]
impl TransformationData for CypherResultDataFrame {
    fn get_context(&self) -> &DataTransformationContext {
        &self.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        Box::new(BuiltInOp::new(
            GAF::Cypher,
            TransformationArgs::new_cypher_args(
                self.query.to_string(),
                Schema::Tabular(
                    // TODO(tatiana): fill schema according to query parsing result
                    TabularSchema {
                        tlabel: None,
                        field_names: Vec::new(),
                        field_types: Vec::new(),
                    },
                ),
            ),
            self.get_context().get_transformation_args().clone(),
        ))
    }

    fn get_func(&self) -> GAF {
        GAF::Cypher
    }
}
