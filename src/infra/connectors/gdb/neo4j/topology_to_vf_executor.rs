use super::{
    input::handle_graph_input, GraphComputationExecutor, GraphProjection, Neo4JQueryRowSource,
    Neo4jDatabaseProvider, Storage, TransformationArgs, PULL_SIZE,
};
use crate::{
    infra::pi::{
        storage::{Schema, TabularSchema},
        transformation::GAF,
    },
    transformation::transformation_args::VertexFeatureTransformationArgs,
    transformation::{
        transformation_args::CypherTransformationArgs, TransformationIOT,
        TransformationOutputHandler,
    },
    FeatureValueType, SeResult,
};
use log::info;
use std::sync::Arc;

/// Executors for algorithms that take in edge data only and compute vertex feature(s)
pub(crate) struct TopologyToVFExecutor {
    args: TransformationArgs,
    db: Arc<Neo4jDatabaseProvider>,
    source_types: Vec<Storage>,
    sink_type: Storage,
    func: GAF,
}

impl TopologyToVFExecutor {
    pub(super) fn new(
        args: TransformationArgs,
        db: Arc<Neo4jDatabaseProvider>,
        source_types: Vec<Storage>,
        sink_type: Storage,
        func: GAF,
    ) -> Self {
        Self {
            db,
            args,
            source_types,
            sink_type,
            func,
        }
    }

    fn get_query(&self, args: &VertexFeatureTransformationArgs, projected_graph: &str) -> String {
        match self.func {
            GAF::BetweennessCentrality => self.betweenness_centrality_query(args, projected_graph),
            GAF::PageRank => self.page_rank_query(args, projected_graph),
            GAF::TriangleCount => self.triangle_count_query(args, projected_graph),
            _ => panic!("Unexpected func"),
        }
    }

    fn page_rank_query(
        &self,
        args: &VertexFeatureTransformationArgs,
        projected_graph: &str,
    ) -> String {
        let algo_args = args.algorithm.as_page_rank();
        // TODO(han): support multiple node labels
        format!(
            "CALL gds.pageRank.stream('{projected_graph}', 
                {{dampingFactor: {}, maxIterations: {}, tolerance: {}}}
            ) YIELD nodeId, score 
            MATCH (n) where ID(n) = nodeId
            RETURN n.{}, score",
            algo_args.damping_factor,
            algo_args.max_iteration,
            algo_args.tolerance,
            args.target_vertex_primary_key
        )
    }

    fn betweenness_centrality_query(
        &self,
        args: &VertexFeatureTransformationArgs,
        projected_graph: &str,
    ) -> String {
        let algo_args = args.algorithm.as_betweenness_centrality();
        format!(
            "CALL gds.betweenness.stream('{projected_graph}',
                       {{samplingSize:{} , samplingSeed:{} }}
                   ) YIELD nodeId, score
                   MATCH (n) where ID(n) = nodeId
                   RETURN n.{}, score",
            if let Some(size) = algo_args.sampling_size {
                size.to_string()
            } else {
                "null".to_string()
            },
            if let Some(seed) = algo_args.sampling_seed {
                seed.to_string()
            } else {
                "null".to_string()
            },
            args.target_vertex_primary_key
        )
    }

    fn triangle_count_query(
        &self,
        args: &VertexFeatureTransformationArgs,
        projected_graph: &str,
    ) -> String {
        let algo_args = args.algorithm.as_triangle_count();
        format!(
            "CALL gds.triangleCount.stream('{projected_graph}'{}
            ) YIELD nodeId, triangleCount
            MATCH (n) where ID(n) = nodeId
            RETURN n.{}, triangleCount",
            if let Some(max_degree) = algo_args.max_degree {
                format!(", {{ maxDegree: {max_degree} }}")
            } else {
                String::new()
            },
            args.target_vertex_primary_key
        )
    }
}

#[async_trait::async_trait]
impl GraphComputationExecutor for TopologyToVFExecutor {
    async fn execute(&self, input: &TransformationIOT) -> SeResult<TransformationOutputHandler> {
        let input_graph = input.first().expect("Input graph is expected");
        if !handle_graph_input(input_graph, &self.db, &self.source_types[0]) {
            return Ok(TransformationOutputHandler::EmptyOutput);
        }

        let args = self.args.as_vertex_feature();
        let projection = GraphProjection::new(&self.db, &args.graph_projection, self.func.clone());
        let projected_graph = match projection.project_graph().await {
            Ok(name) => name,
            Err(error) => {
                info!("Error when projecting graph. {error}");
                // FIXME(han): this is a workaround to make it not crash on empty dataset in CI
                return Ok(TransformationOutputHandler::EmptyOutput);
            }
        };

        let query = self.get_query(args, projected_graph);
        match self.sink_type {
            Storage::OfnilRow => {
                Ok(TransformationOutputHandler::TabularSource(Arc::new(
                    Neo4JQueryRowSource::new(
                        self.db.clone(),
                        CypherTransformationArgs::new(
                            query,
                            Schema::Tabular(TabularSchema {
                                tlabel: Some(args.target_vertex_tlabel.clone()),
                                field_names: args.output_names.clone(),
                                // TODO(tatiana): consider other feature types here?
                                field_types: vec![FeatureValueType::Float; args.output_names.len()],
                            }),
                        ),
                        PULL_SIZE,
                    ),
                )))
            }
            _ => unimplemented!("Now only support in-process row format"),
        }
    }
}
