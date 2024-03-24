mod vertex_feature_transformation_args;
pub use vertex_feature_transformation_args::VertexFeatureTransformationArgs;
mod cypher_transformation_args;
pub use cypher_transformation_args::CypherTransformationArgs;

use super::BuiltInFnArgs;
use crate::{
    feature::EdgeEntity,
    infra::pi::{Schema, Storage},
    Entity, InfraIdentifier,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, enum_methods::EnumAsGetters, enum_methods::EnumIntoGetters)]
pub enum TransformationArgs {
    VertexFeature(VertexFeatureTransformationArgs),
    EdgeFeature,
    Cypher(CypherTransformationArgs),
}

impl TransformationArgs {
    /// Creates a new [`VertexFeatureTransformationArgs`].
    ///
    /// # Parameter
    /// See [`VertexFeatureTransformationArgs`].
    pub fn new_vertex_feature_args(
        algorithm: BuiltInFnArgs,
        graph_projection: GraphProjectionArgs,
        target_vertex_tlabel: String,
        target_vertex_primary_key: String,
        output_names: Vec<String>,
    ) -> Self {
        Self::VertexFeature(VertexFeatureTransformationArgs::new(
            algorithm,
            graph_projection,
            target_vertex_tlabel,
            target_vertex_primary_key,
            output_names,
        ))
    }

    pub fn new_cypher_args(query: String, output_schema: Schema) -> Self {
        Self::Cypher(CypherTransformationArgs::new(query, output_schema))
    }
}

#[derive(Debug, Clone)]
pub struct GraphProjectionArgs {
    /// a vector of (vertex label, primary key) tuples
    pub vertices: Vec<(String, Option<String>)>,
    /// a vector of edge labels
    pub edges: Vec<EdgeEntity>,
    pub make_edges_undirected: bool,
}

impl GraphProjectionArgs {
    /// Creates a new [`GraphProjectionArgs`].
    ///
    /// # Parameter
    /// entities: vertices and edges on which to project a given graph
    /// make_edges_undirected: if true, treat edges as undirected
    pub fn new(entities: &Vec<Entity>, make_edges_undirected: bool) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for entity in entities {
            match entity {
                Entity::Vertex(entity) => {
                    nodes.push((entity.tlabel.clone(), Some(entity.primary_key.to_owned())));
                }
                Entity::Edge(entity) => {
                    edges.push(entity.clone());
                }
            }
        }
        Self {
            vertices: nodes,
            edges,
            make_edges_undirected,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CommonTransformationArgs {
    /// Specifies the infra to execute this transformation operation.
    infra_id: Option<InfraIdentifier>,
    sink_storage_type: Option<Storage>, // required for execution
    source_storage_type: Vec<Storage>,
}

impl CommonTransformationArgs {
    pub fn new(infra_id: Option<InfraIdentifier>) -> Self {
        Self {
            infra_id,
            sink_storage_type: None,
            source_storage_type: Vec::new(),
        }
    }

    pub fn infra_id(&self) -> Option<&InfraIdentifier> {
        self.infra_id.as_ref()
    }

    pub fn set_sink_storage_type(&mut self, sink_type: Storage) {
        self.sink_storage_type = Some(sink_type);
    }

    pub fn set_source_storage_type(&mut self, source_type: Vec<Storage>) {
        self.source_storage_type = source_type;
    }

    pub fn source_storage_types(&self) -> &Vec<Storage> {
        &self.source_storage_type
    }

    pub fn sink_storage_type(&self) -> Option<&Storage> {
        self.sink_storage_type.as_ref()
    }
}
