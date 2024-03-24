use super::{plan_op::PlanOp, Neo4jDatabaseProviderError};
use crate::{
    infra::{connectors::*, pi::*},
    transformation::{GraphSchema, GraphSchemaEntity, InputSchema},
    FeatureValueType,
};

use bb8_bolt::{
    bb8::PooledConnection,
    bolt_client::Metadata,
    bolt_proto::message::{Failure, Message, Success},
    bolt_proto::Value,
    Manager,
};
use log::debug;
use std::error::Error;

// TODO(tatiana): refactor using openCypher parser instead of adhoc neo4j plan parser
pub struct Neo4jQueryParser<'a> {
    query: String,
    // TODO: guarantee correctness as the underlying bolt connection cannot be shared among async queries.
    bolt_conn: PooledConnection<'a, Manager>,
    plan: Option<(String, PlanOp)>, // db, ops
    returned_fields: Option<Vec<String>>,
    failure: Option<(String, String)>, // code, message
}

impl<'a> Neo4jQueryParser<'a> {
    pub async fn new(
        query: &str,
        bolt_conn: PooledConnection<'a, Manager>,
    ) -> Result<Neo4jQueryParser<'a>, Box<dyn Error>> {
        let mut res = Self {
            query: query.to_string(),
            bolt_conn,
            plan: None,
            returned_fields: None,
            failure: None,
        };
        res.parse().await?;
        Ok(res)
    }

    async fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let query = if !self.query.to_lowercase().starts_with("explain") {
            format!("EXPLAIN {}", &self.query)
        } else {
            self.query.clone()
        };
        let message = self.bolt_conn.run(&query, None, None).await?;
        match message {
            Message::Success(success) => {
                if let Some(fields) = success.metadata().get("fields") {
                    match fields {
                        Value::List(fields) => {
                            let mut returned_fields = Vec::new();
                            for field in fields {
                                match field {
                                    Value::String(identifier) => {
                                        returned_fields.push(identifier.to_owned());
                                    }
                                    _ => {
                                        return Err(Box::new(
                                            Neo4jDatabaseProviderError::BoltMessageError(
                                                "Unexpected type of `fields` in Message::Success.",
                                            ),
                                        ));
                                    }
                                }
                            }
                            self.returned_fields = Some(returned_fields);
                        }
                        _ => {
                            return Err(Box::new(Neo4jDatabaseProviderError::BoltMessageError(
                                "Unexpected type of `fields` in Message::Success.",
                            )));
                        }
                    }
                } else {
                    return Err(Box::new(Neo4jDatabaseProviderError::BoltMessageError(
                        "Cannot get fields in Message::Success.",
                    )));
                }
                // pull msg
                let (_, message) = self
                    .bolt_conn
                    .pull(Some(Metadata::from_iter(vec![("n", 1)])))
                    .await?;
                match message {
                    Message::Success(success) => {
                        self.plan = match success.metadata().get("plan") {
                            Some(plan) => {
                                let db = success.metadata().get("db");
                                match db {
                                    Some(Value::String(db_name)) => {
                                        let ops = PlanOp::new(db_name, plan).map_err(|e| {
                                            QueryParserError::ConnectorError(Box::new(e))
                                        })?;
                                        println!("{ops:#?}");
                                        Some((db_name.clone(), ops))
                                    }
                                    _ => {
                                        return Err(Box::new(
                                            Neo4jDatabaseProviderError::BoltMessageError(
                                                "Cannot get db in Message::Success.",
                                            ),
                                        ));
                                    }
                                }
                            }
                            None => {
                                debug!("{:?}", success.metadata());
                                return Err(Box::new(
                                    Neo4jDatabaseProviderError::BoltMessageError(
                                        "Cannot get plan in Message::Success",
                                    ),
                                ));
                            }
                        };
                    }
                    Message::Failure(failure) => {
                        self.parse_error_message(&failure)?;
                        self.bolt_conn.reset().await?;
                    }
                    _ => {
                        Success::try_from(message)?;
                    }
                }
            }
            Message::Failure(failure) => {
                self.parse_error_message(&failure)?;
                self.bolt_conn.reset().await?;
            }
            _ => {
                Success::try_from(message)?;
            }
        }
        Ok(())
    }

    fn parse_error_message(&mut self, failure: &Failure) -> Result<(), Neo4jDatabaseProviderError> {
        if let Some(Value::String(failure_msg)) = failure.metadata().get("message") {
            if let Some(Value::String(failure_code)) = failure.metadata().get("code") {
                self.failure = Some((failure_code.clone(), failure_msg.clone()));
                Ok(())
            } else {
                Err(Neo4jDatabaseProviderError::BoltMessageError(
                    "Cannot get code in Message::Failure.",
                ))
            }
        } else {
            Err(Neo4jDatabaseProviderError::BoltMessageError(
                "Cannot get message in Message::Failure.",
            ))
        }
    }

    fn get_vertex_schema(
        &self,
        identifier_name: &str,
        identifier_map: &IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<GraphSchemaEntity, QueryParserError> {
        if let Some(schema) = identifier_map.get_graph_schema_entity(identifier_name, input_schema)
        {
            Ok(schema)
        } else {
            Err(QueryParserError::ConnectorError(Box::new(
                Neo4jDatabaseProviderError::PlanParseError(
                    "Cannot find returned field from parsed PlanOp tree.",
                ),
            )))
        }
    }

    fn get_edge_schema(
        &self,
        src_identifier: &str,
        dst_identifier: &str,
        identifier_map: &IdentifierMap,
        input_schema: &InputSchema,
    ) -> GraphSchemaEntity {
        if let Some(src) = identifier_map.get_identifier(src_identifier) {
            if let Some(dst) = identifier_map.get_identifier(dst_identifier) {
                if let Some(edge) = identifier_map.get_unique_edge(src.index(), dst.index()) {
                    return edge.get_graph_schema_entity(input_schema, identifier_map);
                }
            }
        }
        GraphSchemaEntity {
            tlabel: None,
            entity_id: None,
            fields: Vec::new(),
        }
    }
}

impl QueryParser for Neo4jQueryParser<'_> {
    fn validate_query(
        &self,
        input_schema: Option<&InputSchema>,
        required_fields: usize,
    ) -> Result<(), QueryParserError> {
        if let Some((code, message)) = &self.failure {
            return Err(QueryParserError::GDBError {
                provider: std::any::type_name::<Neo4jConnector>().to_string(),
                code: code.clone(),
                message: message.clone(),
            });
        }
        if self.returned_fields.as_ref().unwrap().len() < required_fields {
            return Err(QueryParserError::ReturnFieldsError(
                required_fields,
                self.returned_fields.as_ref().unwrap().len(),
            ));
        }
        if let Some(schema) = input_schema {
            let mut identifier_map = IdentifierMap::new();
            self.plan
                .as_ref()
                .unwrap()
                .1
                .parse_input(&mut identifier_map, schema)
                .map_err(|error| QueryParserError::ConnectorError(Box::new(error)))?;
        }
        Ok(())
    }

    fn get_output_graph_schema(
        &self,
        input_schema: &InputSchema,
    ) -> Result<GraphSchema, QueryParserError> {
        let mut identifier_map = IdentifierMap::new();
        let root = &self.plan.as_ref().unwrap().1;
        root.parse_output(&mut identifier_map, input_schema)
            .map_err(|error| QueryParserError::ConnectorError(Box::new(error)))?;
        let fields = self.returned_fields.as_ref().unwrap();
        println!("{identifier_map:#?}");
        let src = self.get_vertex_schema(&fields[0], &identifier_map, input_schema)?;
        let dst = self.get_vertex_schema(&fields[1], &identifier_map, input_schema)?;
        let mut edge = self.get_edge_schema(&fields[0], &fields[1], &identifier_map, input_schema);
        if fields.len() > 2 {
            for field in &fields[2..] {
                if let Some(value_type) = identifier_map
                    .get_identifier(field)
                    .expect("Identifier should have been found during parsing")
                    .prop()
                    .get_type()
                {
                    edge.fields.push((field.clone(), value_type.clone()));
                } else {
                    // TODO(tatiana): parse types
                    edge.fields.push((field.clone(), FeatureValueType::String));
                }
            }
            todo!()
        }
        Ok(GraphSchema { src, dst, edge })
    }
}
