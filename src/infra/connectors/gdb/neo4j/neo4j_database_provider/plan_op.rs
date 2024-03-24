mod aggregation;
mod arg_regex;
mod expand_all;
mod filter;
mod projection;
mod scan;
use bb8_bolt::bolt_client::bolt_proto::Value;
use log::debug;

use std::collections::HashMap;

use super::plan_op_constant::PLAN_OPS;
use super::Neo4jDatabaseProviderError;
use crate::{
    infra::{connectors::IdentifierMap, pi::QueryParserError},
    transformation::InputSchema,
};
use aggregation::*;
use expand_all::ExpandAll;
use filter::Filter;
use projection::Projection;
use scan::*;

#[derive(Debug)]
pub enum PlanOp {
    ProduceResults(PlanOpImpl),
    Projection(PlanOpImpl),
    Filter(PlanOpImpl),
    ExpandAll(PlanOpImpl),
    AllNodesScan(PlanOpImpl),
    NodeByLabelScan(PlanOpImpl),
    DirectedRelationshipTypeScan(PlanOpImpl),
    EagerAggregation(PlanOpImpl),
    IgnoredOp(String, PlanOpImpl),
}

impl PlanOp {
    pub(super) fn new(db: &String, value: &Value) -> Result<PlanOp, Neo4jDatabaseProviderError> {
        if let Value::Map(map) = value {
            match map.get("operatorType") {
                Some(Value::String(name)) => {
                    let op_name = name.split('@').next().unwrap();
                    match op_name {
                        "ProduceResults" => Ok(PlanOp::ProduceResults(PlanOpImpl::new(db, map)?)),
                        "Projection" => Ok(PlanOp::Projection(PlanOpImpl::new(db, map)?)),
                        "Filter" => Ok(PlanOp::Filter(PlanOpImpl::new(db, map)?)),
                        "Expand(All)" => Ok(PlanOp::ExpandAll(PlanOpImpl::new(db, map)?)),
                        "AllNodesScan" => Ok(PlanOp::AllNodesScan(PlanOpImpl::new(db, map)?)),
                        "NodeByLabelScan" => Ok(PlanOp::NodeByLabelScan(PlanOpImpl::new(db, map)?)),
                        "DirectedRelationshipTypeScan" => Ok(PlanOp::DirectedRelationshipTypeScan(
                            PlanOpImpl::new(db, map)?,
                        )),
                        "EagerAggregation" => {
                            Ok(PlanOp::EagerAggregation(PlanOpImpl::new(db, map)?))
                        }
                        "NodeHashJoin" | "CacheProperties" => Ok(PlanOp::IgnoredOp(
                            op_name.to_owned(),
                            PlanOpImpl::new(db, map)?,
                        )),
                        _ => {
                            if PLAN_OPS.contains(&op_name) {
                                println!("{op_name}, {map:?}");
                                todo!()
                            } else {
                                Err(Neo4jDatabaseProviderError::PlanParseError(
                                    "Unknown operatorType",
                                ))
                            }
                        }
                    }
                }
                None => Err(Neo4jDatabaseProviderError::PlanParseError(
                    "No children field in plan op",
                )),
                _ => Err(Neo4jDatabaseProviderError::PlanParseError(
                    "The field operatorType in plan op is not of type String",
                )),
            }
        } else {
            Err(Neo4jDatabaseProviderError::PlanParseError(
                "The plan op is not of type Map",
            ))
        }
    }

    #[inline]
    fn parse_children_input(
        children: &Vec<PlanOp>,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        for child in children {
            child.parse_input(map, input_schema)?;
        }
        Ok(())
    }

    pub(super) fn parse_input(
        &self,
        map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        match &self {
            PlanOp::ProduceResults(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
            }
            PlanOp::Projection(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                Projection::modify_identifiers(&op.args, map)?;
            }
            PlanOp::Filter(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                Filter::modify_identifiers(&op.args, map, input_schema)?;
            }
            PlanOp::ExpandAll(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                ExpandAll::modify_identifiers(&op.args, map, input_schema)?;
            }
            PlanOp::AllNodesScan(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                AllNodesScan::modify_identifiers(&op.args, map)?;
            }
            PlanOp::NodeByLabelScan(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                NodeByLabelScan::modify_identifiers(&op.args, map, input_schema)?;
            }
            PlanOp::DirectedRelationshipTypeScan(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                DirectedRelationshipTypeScan::modify_identifiers(&op.args, map, input_schema)?;
            }
            PlanOp::EagerAggregation(op) => {
                Self::parse_children_input(&op.children, map, input_schema)?;
                EagerAggregation::modify_identifiers(&op.args, map)?;
            }
            PlanOp::IgnoredOp(name, op) => {
                debug!("ignored op {}", name);
                Self::parse_children_input(&op.children, map, input_schema)?;
            } // _ => {
              //     println!("unimplemented {:?}", self);
              //     todo!();
              // }
        }
        Ok(())
    }

    pub(super) fn parse_output(
        &self,
        identifier_map: &mut IdentifierMap,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        match &self {
            PlanOp::ProduceResults(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
            }
            PlanOp::Filter(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                Filter::modify_identifiers(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::Projection(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                Projection::annotate_output_types(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::ExpandAll(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                ExpandAll::modify_identifiers(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::AllNodesScan(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                AllNodesScan::annotate_output_types(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::NodeByLabelScan(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                NodeByLabelScan::modify_identifiers(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::DirectedRelationshipTypeScan(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                DirectedRelationshipTypeScan::modify_identifiers(
                    &op.args,
                    identifier_map,
                    input_schema,
                )?;
            }
            PlanOp::EagerAggregation(op) => {
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
                EagerAggregation::annotate_output_types(&op.args, identifier_map, input_schema)?;
            }
            PlanOp::IgnoredOp(name, op) => {
                debug!("parse_output ignoredop {}", name);
                for child in &op.children {
                    child.parse_output(identifier_map, input_schema)?;
                }
            } // _ => {
              //     println!("unimplemented {:?}", self);
              //     todo!();
              // }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlanOpImpl {
    children: Vec<PlanOp>,
    args: String,
}

impl PlanOpImpl {
    pub(super) fn new(
        db: &String,
        map: &HashMap<String, Value>,
    ) -> Result<Self, Neo4jDatabaseProviderError> {
        let child_ops: Vec<PlanOp> = if let Some(children) = map.get("children") {
            match children {
                Value::List(list) => {
                    let mut vec = Vec::new();
                    for child in list {
                        vec.push(PlanOp::new(db, child)?);
                    }
                    vec
                }
                _ => {
                    return Err(Neo4jDatabaseProviderError::PlanParseError(
                        "The children field in plan op is not of type List",
                    ));
                }
            }
        } else {
            return Err(Neo4jDatabaseProviderError::PlanParseError(
                "No children field in plan op",
            ));
        };
        // let identifiers = map.get("identifiers");
        let args = if let Some(Value::Map(map)) = map.get("args") {
            if let Some(Value::String(details)) = map.get("Details") {
                details.to_owned()
            } else {
                return Err(Neo4jDatabaseProviderError::PlanParseError(
                    "Cannot get Details in args",
                ));
            }
        } else {
            return Err(Neo4jDatabaseProviderError::PlanParseError(
                "Cannot get args in plan op",
            ));
        };
        Ok(Self {
            children: child_ops,
            args,
        })
    }
}
