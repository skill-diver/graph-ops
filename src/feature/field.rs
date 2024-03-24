use pyo3::prelude::*;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::InfraIdentifier;

use super::{Entity, FeatureValueType, Variant};
use super::{ResourceId, ResourceOp};

#[macro_export]
macro_rules! fields {
    ($vec: expr, $fie: expr, $variant: expr, $sink_infra: expr,) => {
        $crate::Field::new_fields($vec, &$fie, $variant, $sink_infra)
    };
}

#[pyclass(get_all)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Field {
    pub name: String,
    pub variant: Variant,
    pub value_type: FeatureValueType,
    pub entity_id: Option<String>, // required for registry
    pub transformation_id: Option<ResourceId>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub owners: Vec<String>,
    pub sink_infra_id: Option<InfraIdentifier>,
}

// Now the Field resource id is set to be `Field/{EntityName}/{FieldName}/{FieldVariant}`
impl ResourceOp for Field {
    fn resource_id(&self) -> ResourceId {
        format!(
            "{}/{}/{}/{}",
            &self.variant,
            "Field",
            Entity::id_to_name(self.entity_id.as_ref().unwrap()),
            &self.name
        )
    }

    fn id_to_name(id: &str) -> &str {
        id.rsplit_once('/').unwrap().0.rsplit_once('/').unwrap().1
    }

    fn sink_infra_id(&self) -> Option<InfraIdentifier> {
        self.sink_infra_id.clone()
    }

    fn transformation_id(&self) -> Option<ResourceId> {
        self.transformation_id.clone()
    }
}

impl Field {
    pub fn new_fields(
        name_values: Vec<(&str, FeatureValueType)>,
        entity: &Entity,
        variant: Variant,
        sink_infra_id: Option<InfraIdentifier>,
    ) -> Vec<Field> {
        name_values
            .iter()
            .map(|name_type| Field {
                name: name_type.0.to_string(),
                variant: variant.clone(),
                value_type: name_type.1.clone(),
                entity_id: Some(entity.resource_id()),
                transformation_id: None,
                description: None,
                tags: HashMap::new(),
                owners: Vec::new(),
                sink_infra_id: sink_infra_id.clone(),
            })
            .collect()
    }
}

#[pyfunction]
pub(crate) fn fields(
    name_types: Vec<(String, FeatureValueType)>,
    entity: Entity,
    variant: Option<String>,
    sink_infra_id: Option<InfraIdentifier>,
) -> PyResult<Vec<Field>> {
    let mut name_type_tuples = Vec::new();
    name_type_tuples.reserve(name_types.len());
    for (name, vtype) in &name_types {
        name_type_tuples.push((
            name.as_str(),
            vtype.to_owned(), // FeatureValueType::from_str(vtype).map_err(|e| PyValueError::new_err(e.to_string()))?,
        ));
    }
    Ok(Field::new_fields(
        name_type_tuples,
        &entity,
        match variant {
            Some(str) => Variant::UserDefined(str),
            None => Variant::Default(),
        },
        sink_infra_id,
    ))
}
