use super::*;
use regex::Captures;

pub(super) struct Projection;

impl Projection {
    fn get_as_name(captures: &Captures) -> Result<String, QueryParserError> {
        if let Some(new_identifier) = captures.name("name1") {
            Ok(new_identifier.as_str().to_string())
        } else if let Some(new_identifier) = captures.name("name2") {
            Ok(new_identifier.as_str().to_string())
        } else {
            println!("{captures:#?}");
            Err(QueryParserError::ConnectorError(Box::new(
                Neo4jDatabaseProviderError::PlanParseError(
                    "Unexpected Projection args. Cannot parse AS name.",
                ),
            )))
        }
    }

    fn get_as_expression<'a>(captures: &'a Captures<'a>) -> Result<&'a str, QueryParserError> {
        if let Some(expr) = captures.name("expr") {
            Ok(expr.as_str())
        } else {
            println!("{:#?}, {}", captures, arg_regex::PROJECTION.as_str());
            Err(QueryParserError::ConnectorError(Box::new(
                Neo4jDatabaseProviderError::PlanParseError(
                    "Unexpected Projection args. Cannot parse expression.",
                ),
            )))
        }
    }

    pub(super) fn modify_identifiers(
        args: &str,
        map: &mut IdentifierMap,
    ) -> Result<(), QueryParserError> {
        for captures in arg_regex::PROJECTION.captures_iter(args) {
            let new_identifier_name = Self::get_as_name(&captures)?;
            let expr = Self::get_as_expression(&captures)?;
            // TODO(tatiana): support binary operators like +-*/
            for expr_captures in arg_regex::SIMPLE_EXPRESSION.captures_iter(expr) {
                if let Some(name) = expr_captures.name("name") {
                    if name.as_str().starts_with('$') {
                        todo!(); // constant value; continue
                    }
                    if let Some(prop) = expr_captures.name("prop") {
                        let origin = map
                            .get_identifier(name.as_str())
                            .expect("identifier should be in map");
                        origin.check_needed_property(name.as_str(), prop.as_str())?;
                        map.add_prop_identifier(&new_identifier_name, origin.index());
                    } else if let Some(func) = expr_captures.name("func") {
                        let origin = map
                            .get_identifier(name.as_str())
                            .expect("identifier should be in map");
                        if func.as_str() == "ID" {
                            map.add_id_identifier(&new_identifier_name, origin.index());
                        }
                    }
                } else {
                    println!("{captures:#?}");
                    return Err(QueryParserError::ConnectorError(Box::new(
                        Neo4jDatabaseProviderError::PlanParseError(
                            "Unexpected Projection args. Cannot parse expression field.",
                        ),
                    )));
                }
            }
        }
        Ok(())
    }

    pub(super) fn annotate_output_types(
        args: &str,
        identifier_map: &mut IdentifierMap,
        _input_schema: &crate::transformation::InputSchema,
    ) -> Result<(), QueryParserError> {
        for captures in arg_regex::PROJECTION.captures_iter(args) {
            let new_identifier_name = Self::get_as_name(&captures)?;
            let expr = Self::get_as_expression(&captures)?;
            println!("expr {expr:#?} name {new_identifier_name}");
            let mut simple_expression_count = 0;
            for expr_captures in arg_regex::SIMPLE_EXPRESSION.captures_iter(expr) {
                // FIXME(tatiana): support binary operators like +-*/
                if simple_expression_count == 1 {
                    panic!("expression with binary operators is not supported yet");
                } else {
                    simple_expression_count += 1;
                }
                println!(
                    "func {:#?}, expr {:#?}",
                    expr_captures.name("func"),
                    expr_captures
                );
                if let Some(name) = expr_captures.name("name") {
                    if name.as_str().starts_with('$') {
                        todo!(); // constant value; continue;
                    }
                    if let Some(prop) = expr_captures.name("prop") {
                        let origin = identifier_map
                            .get_identifier(name.as_str())
                            .expect("identifier should be in map");
                        if expr_captures.name("func").is_none() {
                            let feature_type = origin
                                .check_needed_property(name.as_str(), prop.as_str())?
                                .clone();
                            identifier_map
                                .add_prop_identifier(&new_identifier_name, origin.index())
                                .set_type(feature_type);
                        }
                    } else if let Some(func) = expr_captures.name("func") {
                        let origin = identifier_map
                            .get_identifier(name.as_str())
                            .expect("identifier should be in map");
                        println!("func {}, map {:#?}", func.as_str(), identifier_map);
                        if func.as_str() == "ID" {
                            identifier_map.add_id_identifier(&new_identifier_name, origin.index());
                        }
                    } else {
                        identifier_map.rename_identifier(name.as_str(), &new_identifier_name);
                    }
                } else {
                    println!("{captures:#?}");
                    return Err(QueryParserError::ConnectorError(Box::new(
                        Neo4jDatabaseProviderError::PlanParseError(
                            "Unexpected Projection args. Cannot parse expression field.",
                        ),
                    )));
                }
            }
        }
        Ok(())
    }
}
