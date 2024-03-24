use crate::{
    transformation::{built_in_fns::expression::Expression, DataIdT},
    FeatureValueType, Field, InfraIdentifier, Variant,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    /// The origin DataFrame
    pub origin: DataIdT,
    /// The string expression used to compute the column from the origin
    pub expr: Option<String>,
    /// The expression encoder for evaluation
    pub encoder: Option<Expression>,
    /// The column value type
    pub value_type: FeatureValueType,
}

impl Column {
    pub fn new(origin: DataIdT, value_type: FeatureValueType) -> Self {
        Column {
            origin,
            expr: None,
            encoder: None,
            value_type,
        }
    }

    pub fn to_field(
        &self,
        name: &str,
        transformation_id: String,
        entity_id: Option<String>,
        sink_infra_id: Option<&InfraIdentifier>,
    ) -> Field {
        Field {
            name: name.to_string(),
            variant: Variant::Default(),
            value_type: self.value_type.clone(),
            // FIXME(tatiana): how should we determine the entity? trace back input column? or let user assign by function parameter?
            entity_id,
            transformation_id: Some(transformation_id),
            description: None,
            tags: HashMap::new(),
            owners: Vec::new(),
            sink_infra_id: sink_infra_id.cloned(),
        }
    }
}
