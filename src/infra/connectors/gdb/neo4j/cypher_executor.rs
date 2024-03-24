use super::{
    input::handle_graph_input, neo4j_database_provider::Neo4jDatabaseProvider, Neo4JQueryRowSource,
    PULL_SIZE,
};
use crate::{
    infra::pi::{storage::Storage, transformation::GraphComputationExecutor},
    transformation::{
        transformation_args::CypherTransformationArgs, TransformationArgs, TransformationIOT,
        TransformationOutputHandler,
    },
    SeResult,
};
use std::sync::Arc;

pub(super) struct CypherExecutor {
    db: Arc<Neo4jDatabaseProvider>,
    source_types: Vec<Storage>,
    sink_type: Storage,
    args: CypherTransformationArgs,
}
impl CypherExecutor {
    pub(super) fn new(
        args: TransformationArgs,
        db: Arc<Neo4jDatabaseProvider>,
        source_types: Vec<Storage>,
        sink_type: Storage,
    ) -> Self {
        Self {
            db,
            source_types,
            sink_type,
            args: args.into_cypher(),
        }
    }
}

#[async_trait::async_trait]
impl GraphComputationExecutor for CypherExecutor {
    async fn execute(&self, input: &TransformationIOT) -> SeResult<TransformationOutputHandler> {
        if !handle_graph_input(input.first().unwrap(), &self.db, &self.source_types[0]) {
            return Ok(TransformationOutputHandler::EmptyOutput);
        }
        match self.sink_type {
            Storage::OfnilRow => Ok(TransformationOutputHandler::TabularSource(Arc::new(
                Neo4JQueryRowSource::new(self.db.clone(), self.args.clone(), PULL_SIZE),
            ))),
            _ => unimplemented!("Now only support in-process row data"),
        }
    }
}
