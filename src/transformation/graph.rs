mod multiple_graphs;
mod select;
mod single_graph;

pub use select::{DataFrameSet, EdgeSelectGraph, Selector, VertexSelectGraph};
pub use single_graph::SingleGraph;

use super::{
    built_in_fns::{
        betweenness_centrality_args::BetweennessCentralityArgs, page_rank_args::PageRankArgs,
        triangle_count_args::TriangleCountArgs,
    },
    dataframes::{
        AggregateDataFrame, AggregateFunc, Column, DataFrameInner, VertexFeatureDataFrame,
    },
    random_walk::RandomWalkPath,
    BuiltInFnArgs, CommonTransformationArgs, TransformationData,
};
use crate::{
    feature::ResourceId, Entity, FeatureValueType, Field, InfraIdentifier, Topology, TopologyType,
};
use std::{collections::HashMap, error::Error, rc::Rc, str::FromStr};

pub trait GraphBase: TransformationData {
    fn get_vertex_fvs(&self) -> &HashMap<String, (String, Vec<Field>)>;
    fn get_edge_fvs(&self) -> &HashMap<String, (String, Vec<Field>)>;
    fn get_vertex_entities(&self) -> &HashMap<String, ResourceId>;
    fn get_edge_entities(&self) -> &HashMap<String, ResourceId>;
    fn get_topology_type(&self) -> &Option<TopologyType>;

    /// Returns a vertex data frame containing all vertices in the graph
    fn vertices(&self) -> Rc<dyn GraphBase>;

    /// Returns an edge data frame containing all edges in the graph
    fn edges(&self) -> Rc<dyn GraphBase>;

    /// Returns a vertex data frame containing all vertices of the given type in the graph
    ///
    /// # Arguments
    /// * `t` - The type of vertices to return. If the type does not exist, an error is raised
    fn vertices_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>>;

    /// Returns a edge data frame containing all edges of the given type in the graph
    ///
    /// # Arguments
    /// * `t` - The type of edges to return. If the type does not exist, an error is raised
    fn edges_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>>;

    fn export_topology(&self, name: &str, sink_infra_id: &InfraIdentifier) -> Topology;

    fn export_df(&self, _sink_infra_id: &InfraIdentifier) -> Vec<Field> {
        Vec::<Field>::new()
    }

    fn export(&self, name: &str, sink_infra_id: &InfraIdentifier) -> (Topology, Vec<Field>) {
        (
            self.export_topology(name, sink_infra_id),
            self.export_df(sink_infra_id),
        )
    }
}

pub trait MultipleGraphsBase {
    // TODO(tatiana): fn sample_graphs
}

// TODO(han): input parameters validation
pub trait GraphComputationOps {
    /// Returns an induced subgraph of the graph containing only the given vertices
    ///
    /// # Arguments
    /// * `vertices` - The vertex set from which the induced subgraph is computed
    // TODO(tatiana): interface for subgraph extraction?
    // fn subgraph(&self, vertices: ???) -> Rc<Self>;

    /// Returns a vertex data frame of vertices visited by the random walks. TODO(tatiana) need to consider the return type
    ///
    /// # Arguments
    ///
    /// * `path` - The random walk path length and type specification
    /// * `prob` - The name of the edge feature to use as the transition probability
    /// * `restart_prob` - Probability to terminate the current trace before each transition. If None, the probability is 0
    fn random_walk(
        &self,
        path: RandomWalkPath,
        prob: &str,
        restart_prob: Option<&Vec<f32>>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase>;

    fn random_walk_edges(
        &self,
        path: RandomWalkPath,
        prob: &str,
        restart_prob: Option<&Vec<f32>>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase>;

    /// Returns a vertex data frame with the same set of vertices but new vertex features computed from neighbor aggregation
    ///
    /// # Arguments
    ///
    /// * `edge_entity` - The type of edges to traverse. If None, all edge types are traversed as if in a homogeneous graph
    /// * `target_node_entity` - Determine the direction of the aggregation. It specifies the aggregation destination. It specifies the primary key of the output dataframe.
    /// * `properties` - The name of the destination node feature to use, if `len(property)=0`, then aggregate all the properties in desitination vertex.
    /// * `aggregator` - The aggregator to use for aggregating the neighbor features
    // TODO(han): interface for handling ambiguous edges
    fn aggregate_neighbors(
        &self,
        edge_entity: Option<Entity>, // edge entity
        target_node_entity: Entity, // determine the target node entity, which is the aggregation destination
        properties: Vec<String>,    // or Vec<Field>?
        aggregator: &str,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<AggregateDataFrame>, Box<dyn Error>>;

    /// Returns a vertex data frame with the same set of vertices but new vertex features computed from k-hop neighbor
    /// aggregation
    ///
    /// # Arguments
    ///
    /// * `k` - The number of hops to traverse and aggregate
    /// * `edge_types` - The type of edges to traverse for each hop. If the vector is empty, all edge types are traversed
    ///  as if in a homogeneous graph. If only one edge type is given, it is used for all hops. If multiple edge types are
    ///  given, the number of edge types must be equal to the number of hops
    /// * `aggregator` - The aggregator to use for aggregating the neighbor features for each hop. If only one aggregator
    ///  is given, it is used for all hops. If multiple aggregators are given, the number of aggregators must be equal to
    ///  the number of hops
    /// * `output_col_name` - The name of the output column
    fn aggregate_k_hop_neighbors(
        &self,
        k: u32,
        edge_types: Vec<String>,
        aggregator: &str,
        output_col_name: String,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase>;

    /// Samples a fixed number of neighbors for each vertex in the data frame, and returns the vertices with their sampled
    /// neighbors as a graph
    ///
    /// # Arguments
    ///
    /// * `fanout` - The number of neighbors to sample for each vertex
    /// * `edge_type` - The type of edges to traverse. If None, all edge types are traversed as if in a homogeneous graph
    /// * `replace` - Whether to sample with replacement
    fn sample_neighbors(
        &self,
        fanout: u32,
        edge_type: Option<String>,
        replace: bool,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase>;

    /// Samples a fixed number of k-hop neighbors for each vertex in the data frame, and returns the vertices with their sampled
    /// neighbors as a graph
    ///
    /// # Arguments
    ///
    /// * `k` - The number of hops to traverse and aggregate
    /// * `fanouts` - The number of neighbors to sample for vertices in each hop. If only one fanout is given, it is used for all
    /// hops. If multiple fanouts are given, the number of fanouts must be equal to the number of hops
    /// * `edge_types` - The type of edges to traverse for each hop. If None, all edge types are traversed as if in a homogeneous
    /// graph. If only one edge type is given, it is used for all hops. If multiple edge types are given, the number of edge
    /// types must be equal to the number of hops
    /// * `replace` - Whether to sample with replacement
    fn sample_k_hop_neighbors(
        &self,
        k: u32,
        fanouts: Vec<u32>,
        edge_types: Option<Vec<String>>,
        replace: bool,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase>;

    fn page_rank(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        damping_factor: Option<f32>,
        max_iteration: Option<u32>,
        tolerance: Option<f32>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>>;

    fn betweenness_centrality(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        sampling_size: Option<u32>,
        sampling_seed: Option<u32>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>>;

    fn triangle_count(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>>;
}

impl<T: GraphBase> GraphComputationOps for T {
    #[allow(unused)]
    fn random_walk(
        &self,
        path: RandomWalkPath,
        prob: &str,
        restart_prob: Option<&Vec<f32>>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase> {
        todo!()
    }

    #[allow(unused)]
    fn random_walk_edges(
        &self,
        path: super::random_walk::RandomWalkPath,
        prob: &str,
        restart_prob: Option<&Vec<f32>>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase> {
        todo!()
    }

    fn aggregate_neighbors(
        &self,
        edge_entity: Option<Entity>, // edge entity
        target_node_entity: Entity,  // specify the entity of the output dataframe.
        properties: Vec<String>,     // or Vec<Field>?
        aggregator: &str,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<AggregateDataFrame>, Box<dyn Error>> {
        let new_data_context = self.get_context().new_data_context(common_args);
        let name = format!("aggregate_neighbors_{}", new_data_context.id);
        let res = Rc::new(AggregateDataFrame::new(
            DataFrameInner::new(
                &name,
                new_data_context,
                (0..(properties.len()))
                    .map(|_| Rc::new(Column::new(self.get_data_id(), FeatureValueType::Float)))
                    .collect(),
                properties
                    .iter()
                    .map(|p| format!("{}_{}", &name, p))
                    .collect(),
            ),
            AggregateFunc::from_str(aggregator)?,
            edge_entity,
            target_node_entity,
            properties,
        ));
        self.get_context().register_data(&res);
        Ok(res)
    }

    #[allow(unused)]
    fn aggregate_k_hop_neighbors(
        &self,
        k: u32,
        edge_types: Vec<String>,
        aggregator: &str,
        output_col_name: String,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase> {
        todo!()
    }

    #[allow(unused)]
    fn sample_neighbors(
        &self,
        fanout: u32,
        edge_type: Option<String>,
        replace: bool,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase> {
        todo!()
    }

    #[allow(unused)]
    fn sample_k_hop_neighbors(
        &self,
        k: u32,
        fanouts: Vec<u32>,
        edge_types: Option<Vec<String>>,
        replace: bool,
        common_args: Option<CommonTransformationArgs>,
    ) -> Rc<dyn GraphBase> {
        todo!()
    }

    // TODO(tatiana): allow configuring output col names?
    fn page_rank(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        damping_factor: Option<f32>,
        max_iteration: Option<u32>,
        tolerance: Option<f32>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>> {
        let new_data_context = self.get_context().new_data_context(common_args);
        let name = format!("page_rank_{}", new_data_context.id);
        let res = Rc::new(VertexFeatureDataFrame::new(
            name.clone(),
            new_data_context,
            vec![Rc::new(Column::new(
                self.get_data_id(),
                FeatureValueType::Float,
            ))],
            vec![name],
            target_node_entity,
            BuiltInFnArgs::PageRank(PageRankArgs {
                damping_factor: damping_factor.unwrap_or(0.85),
                max_iteration: max_iteration.unwrap_or(20),
                tolerance: tolerance.unwrap_or(1e-7),
            }),
            (entities, false),
        ));
        self.get_context().register_data(&res);
        Ok(res)
    }

    fn betweenness_centrality(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        sampling_size: Option<u32>,
        sampling_seed: Option<u32>,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>> {
        let new_data_context = self.get_context().new_data_context(common_args);
        let name = format!("betweenness_centrality_{}", new_data_context.id);
        let res = Rc::new(VertexFeatureDataFrame::new(
            name.clone(),
            new_data_context,
            vec![Rc::new(Column::new(
                self.get_data_id(),
                FeatureValueType::Float,
            ))],
            vec![name],
            target_node_entity,
            BuiltInFnArgs::BetweennessCentrality(BetweennessCentralityArgs {
                sampling_size,
                sampling_seed,
            }),
            (entities, false),
        ));
        self.get_context().register_data(&res);
        Ok(res)
    }

    // TODO(tatiana): algorithm configs
    fn triangle_count(
        &self,
        entities: Vec<Entity>,
        target_node_entity: Entity,
        common_args: Option<CommonTransformationArgs>,
    ) -> Result<Rc<VertexFeatureDataFrame>, Box<dyn Error>> {
        let new_data_context = self.get_context().new_data_context(common_args);
        let name = format!("triangle_count_{}", new_data_context.id);
        let res = Rc::new(VertexFeatureDataFrame::new(
            name.clone(),
            new_data_context,
            vec![Rc::new(Column::new(
                self.get_data_id(),
                FeatureValueType::Float,
            ))],
            vec![name],
            target_node_entity,
            BuiltInFnArgs::TriangleCount(TriangleCountArgs { max_degree: None }),
            // TODO(tatiana): find a better logic. now neo4j requires undirected edges for triangle counting
            (entities, true),
        ));
        self.get_context().register_data(&res);
        Ok(res)
    }
}
