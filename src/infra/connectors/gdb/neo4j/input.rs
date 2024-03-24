use std::sync::Arc;

use crate::{
    infra::pi::storage::Storage, transformation::TransformationOutputHandler, InfraIdentifier,
};

use super::neo4j_database_provider::Neo4jDatabaseProvider;

pub(super) fn handle_graph_input(
    input: &TransformationOutputHandler,
    _db: &Arc<Neo4jDatabaseProvider>,
    source_type: &Storage,
) -> bool {
    match input {
        TransformationOutputHandler::EmptyOutput => {
            return false;
        }
        TransformationOutputHandler::InfraHandler { infra_id } => {
            assert!(
                matches!(infra_id, InfraIdentifier::Neo4j(_)),
                "Input is expected to be in Neo4j, but got {infra_id:?}"
            );
        }
        // TODO(tatiana): If input is topology source, load data into database
        _ => {
            unimplemented!("unexpected input {input:?} of type {:?}", source_type)
        }
    }
    true
}
