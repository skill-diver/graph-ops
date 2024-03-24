use super::*;

pub(super) struct Filter;

impl Filter {
    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        for capture in arg_regex::LABEL_PREDICATE.captures_iter(args) {
            let identifier = &capture[1];
            let label = &capture[2];
            if let Some(id) = map.get_identifier_mut(identifier) {
                id.set_label_schema(label, identifier, input_schema)?;
            } else {
                return Err(QueryParserError::ConnectorError(Box::new(
                    Neo4jDatabaseProviderError::PlanParseError(
                        "Unexpected Filter args, identifier is unknown",
                    ),
                )));
            }
        }
        for capture in arg_regex::PROPERTY_PREDICATE.captures_iter(args) {
            let identifier = &capture[1];
            let prop = &capture[2];
            if let Some(id) = map.get_identifier(identifier) {
                id.check_needed_property(identifier, prop)?;
            } else {
                return Err(QueryParserError::ConnectorError(Box::new(
                    Neo4jDatabaseProviderError::PlanParseError(
                        "Unexpected Filter args, identifier is unknown",
                    ),
                )));
            }
        }
        Ok(())
    }
}
