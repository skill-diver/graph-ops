pub mod expression;
pub mod random_walk;
pub mod sampling;

pub mod aggregate_neighbor_args;
pub mod betweenness_centrality_args;
pub mod page_rank_args;
pub mod triangle_count_args;

use serde::{Deserialize, Serialize};

use crate::infra::pi::GAF;
use std::collections::HashMap;

#[cfg_attr(
    feature = "dashboard",
    derive(strum::EnumString),
    strum(serialize_all = "snake_case")
)]
#[derive(Debug, Clone, enum_methods::EnumAsGetters, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInFnArgs {
    AggregateNeighbor(aggregate_neighbor_args::AggregateNeighborArgs),
    BetweennessCentrality(betweenness_centrality_args::BetweennessCentralityArgs),
    PageRank(page_rank_args::PageRankArgs),
    TriangleCount(triangle_count_args::TriangleCountArgs),
    Custom(HashMap<String, String>),
}

impl BuiltInFnArgs {
    pub fn get_func(&self) -> GAF {
        match self {
            BuiltInFnArgs::AggregateNeighbor(_) => GAF::AggregateNeighbors,
            BuiltInFnArgs::BetweennessCentrality(_) => GAF::BetweennessCentrality,
            BuiltInFnArgs::PageRank(_) => GAF::PageRank,
            BuiltInFnArgs::TriangleCount(_) => GAF::TriangleCount,
            BuiltInFnArgs::Custom(_) => panic!("not built-in function args"),
        }
    }
}
