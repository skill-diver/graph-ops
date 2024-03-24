pub mod gdb;

use crate::transformation::{TransformationIOT, TransformationOutputHandler};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(EnumString, Display, Debug, PartialEq, EnumIter, Clone, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum GraphAnalyticFunc {
    // sourceable, direct retrieval
    #[strum(disabled)]
    Source,
    // graph query support
    Cypher,
    // built-in algorithm support
    AggregateNeighbors,
    BetweennessCentrality,
    BFS,
    PageRank,
    TriangleCount,
    ApproximateClosenessCentrality,
    ArticleRank,
    ClosenessCentrality,
    DegreeCentrality,
    EigenvectorCentrality,
    HarmonicCentrality,
    InfluenceMaximization,
    PersonalizedPageRank,
    WeightedDegreeCentrality,
    WeightedPageRank,
    GreedyGraphColoring,
    KNearestNeighbors,
    MaximalIndependentSet,
    WeaklyConnectedComponents,
    KCoreDecomposition,
    LabelPropagation,
    LocalClusteringCoefficient,
    Louvain,
    StronglyConnectedComponents,
    AdamicAdar,
    CommonNeighbors,
    PreferentialAttachment,
    ResourceAllocation,
    SameCommunity,
    TotalNeighbors,
    AStar,
    AllPairsShortestPath,
    BreadthFirstSearch,
    CycleDetection,
    EstimatedDiameter,
    MaximumFlow,
    MinimumSpanningForest,
    MinimumSpanningTree,
    EuclideanDistance,
    OverlapSimilarity,
    PearsonSimilarity,
    #[strum(disabled)]
    OneOf(Vec<GraphAnalyticFunc>),
}
pub use GraphAnalyticFunc as GAF;

#[async_trait::async_trait]
pub trait GraphComputationExecutor {
    async fn execute(
        &self,
        input: &TransformationIOT,
    ) -> Result<TransformationOutputHandler, Box<dyn std::error::Error>>;
}
