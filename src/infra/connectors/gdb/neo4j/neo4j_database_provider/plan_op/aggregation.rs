use crate::infra::pi::QueryParserError;

use super::IdentifierMap;

pub(super) struct EagerAggregation;

impl EagerAggregation {
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
        input_schema: &crate::transformation::InputSchema,
    ) -> Result<(), QueryParserError> {
        println!("args {args} identifiers {identifier_map:?} input {input_schema:?}");
        todo!()
    }
}
