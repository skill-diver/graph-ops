mod neo4j_database_provider;

use neo4j_database_provider::{Neo4jDatabaseProvider, Neo4jDatabaseProviderError};
mod neo4j_edge_sink;
use neo4j_edge_sink::*;
mod neo4j_query_row_source;
use neo4j_query_row_source::*;
mod graph_projection;
use graph_projection::GraphProjection;
mod topology_to_vf_executor;
use topology_to_vf_executor::TopologyToVFExecutor;
mod cypher_executor;
use cypher_executor::CypherExecutor;
mod graph_csv_sink;
mod input;

use crate::{
    infra::{pi::storage::*, pi::*},
    transformation::{GraphProjectionArgs, TransformationArgs},
    InfraIdentifier, SeResult,
};
use std::sync::Arc;

const PULL_SIZE: i32 = 1024;

#[derive(Debug, Clone)]
pub struct Neo4jConnector {
    inner: Arc<Neo4jDatabaseProvider>,
}

impl Neo4jConnector {
    pub fn get_database(&self) -> Arc<Neo4jDatabaseProvider> {
        self.inner.clone()
    }

    pub(crate) async fn new(
        bolt_uri: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        max_pool_size: Option<u32>,
        sink_infra_id: Option<InfraIdentifier>,
    ) -> SeResult<Self> {
        Ok(Self {
            inner: Arc::new(
                Neo4jDatabaseProvider::new(
                    bolt_uri,
                    username,
                    password,
                    max_pool_size,
                    sink_infra_id,
                )
                .await?,
            ),
        })
    }
}

impl TransformationConnector for Neo4jConnector {
    fn get_supported_funcs(&self) -> Vec<GAF> {
        vec![
            GAF::BetweennessCentrality,
            GAF::Cypher,
            GAF::PageRank,
            GAF::TriangleCount,
        ]
    }

    fn get_graph_executor(
        &self,
        func: &GAF,
        args: TransformationArgs,
        source_type: Vec<Storage>,
        sink_type: Storage,
    ) -> Box<dyn GraphComputationExecutor> {
        assert!(self.supports_func(func));
        match func {
            GAF::Cypher => Box::new(CypherExecutor::new(
                args,
                self.inner.clone(),
                source_type,
                sink_type,
            )),
            GAF::BetweennessCentrality | GAF::PageRank | GAF::TriangleCount => {
                Box::new(TopologyToVFExecutor::new(
                    args,
                    self.inner.clone(),
                    source_type,
                    sink_type,
                    func.clone(),
                ))
            }
            _ => panic!("Func is claimed to be supported but not registered"),
        }
    }
}

impl Sinkable for Neo4jConnector {
    fn get_supported_sources(&self) -> Vec<Storage> {
        vec![Storage::Neo4j, Storage::OfnilRow]
    }

    fn get_sink(&self, src_storage: &Storage, type_info: Schema) -> SinkType {
        match src_storage {
            Storage::OfnilRow => match type_info {
                Schema::Edge(edge_chema) => SinkType::Row(Box::new(Neo4jEdgeSink::new(
                    self.get_database(),
                    edge_chema,
                ))),
                _ => todo!("load vertex/edge properties"),
            },
            _ => unimplemented!("{src_storage:?} is not supported"),
        }
    }
}

impl Sourceable for Neo4jConnector {
    fn get_supported_sinks(&self) -> Vec<Storage> {
        vec![Storage::Neo4j, Storage::OfnilRow]
    }
}
