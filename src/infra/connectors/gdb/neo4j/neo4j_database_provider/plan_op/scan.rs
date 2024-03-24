use super::{arg_regex, IdentifierMap, Neo4jDatabaseProviderError};
use crate::{infra::pi::QueryParserError, transformation::InputSchema};

pub(super) struct AllNodesScan;
pub(super) struct NodeByLabelScan;
pub(super) struct DirectedRelationshipTypeScan;

impl AllNodesScan {
    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
    ) -> Result<(), QueryParserError> {
        println!("{args}, {map:?}");
        todo!()
    }

    pub(super) fn annotate_output_types(
        args: &str,
        identifier_map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        println!("args {args} identifiers {identifier_map:?} input {input_schema:?}");
        todo!()
    }
}

impl NodeByLabelScan {
    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        if let Some(captures) = arg_regex::LABEL_PREDICATE.captures_iter(args).next() {
            let identifier = map.add_vertex_identifier(&captures[1]);
            identifier.set_label_schema(&captures[2], &captures[1], input_schema)
        } else {
            Err(QueryParserError::ConnectorError(Box::new(
                Neo4jDatabaseProviderError::PlanParseError("Unexpected NodeByLabelScan args."),
            )))
        }
    }
}

impl DirectedRelationshipTypeScan {
    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        if let Some(captures) = arg_regex::DIRECTED_EDGE_TYPE.captures_iter(args).next() {
            map.add_vertex_identifier(&captures[1]);
            map.add_vertex_identifier(&captures[4]);
            map.add_edge_identifier(&captures[2], &captures[1], &captures[4])
                .set_label_schema(&captures[3], &captures[2], input_schema)
        } else {
            Err(QueryParserError::ConnectorError(Box::new(
                Neo4jDatabaseProviderError::PlanParseError("Unexpected NodeByLabelScan args."),
            )))
        }
    }
}
