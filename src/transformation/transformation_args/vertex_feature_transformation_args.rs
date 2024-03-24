use super::{BuiltInFnArgs, GraphProjectionArgs};

#[derive(Debug, Clone)]
pub struct VertexFeatureTransformationArgs {
    /// algorithm parameters
    pub algorithm: BuiltInFnArgs,
    /// project graph by vertices and edges
    pub graph_projection: GraphProjectionArgs,
    pub target_vertex_tlabel: String,
    pub target_vertex_primary_key: String,
    /// target vertex feature names
    pub output_names: Vec<String>,
}

impl VertexFeatureTransformationArgs {
    pub fn new(
        algorithm: BuiltInFnArgs,
        graph_projection: GraphProjectionArgs,
        target_vertex_tlabel: String,
        target_vertex_primary_key: String,
        output_names: Vec<String>,
    ) -> Self {
        Self {
            algorithm,
            graph_projection,
            target_vertex_tlabel,
            target_vertex_primary_key,
            output_names,
        }
    }
}
