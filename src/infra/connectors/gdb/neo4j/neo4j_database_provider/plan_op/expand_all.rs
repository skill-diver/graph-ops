use super::*;

pub(super) struct ExpandAll;

impl ExpandAll {
    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        let mut res = arg_regex::EXPAND.captures_iter(args);
        let res = res.next().unwrap();
        let from = &res[1];
        let relationship_name = &res[2];
        let relationship_label = res.get(3);
        let to = &res[4];
        map.get_identifier(from).ok_or_else(|| {
            QueryParserError::ConnectorError(Box::new(Neo4jDatabaseProviderError::PlanParseError(
                "Unexpected args for ExpandAll. The from identifier is unknown",
            )))
        })?;
        map.add_vertex_identifier(to);
        let relationship = map.add_edge_identifier(relationship_name, from, to);
        if let Some(m) = relationship_label {
            relationship.set_label_schema(&m.as_str()[1..], relationship_name, input_schema)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ExpandAll, IdentifierMap, InputSchema, QueryParserError};
    use std::collections::HashMap;

    fn populate_test_input_schema() -> InputSchema {
        let mut res = InputSchema {
            vertex_entities: HashMap::new(),
            edge_entities: HashMap::new(),
        };
        res.edge_entities.insert(
            "belongsTo".to_string(),
            (
                // dummy entity id
                "default/Entity/belongsTo/Product/Category".to_string(),
                HashMap::new(),
            ),
        );
        res
    }

    #[test]
    fn test_modify_identifiers() -> Result<(), QueryParserError> {
        let mut map = IdentifierMap::new();
        let schema = populate_test_input_schema();
        ExpandAll::modify_identifiers("(cat)<-[anon_0]-(p)", &mut map, &schema)
            .expect_err("expected error as cat is not in the map");
        map.add_vertex_identifier("cat");
        ExpandAll::modify_identifiers("(cat)<-[anon_0:belongsTo]-(p)", &mut map, &schema)?;
        println!("{map:#?}");
        Ok(())
    }
}
