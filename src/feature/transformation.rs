use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{ResourceId, ResourceOp};
use crate::{transformation::DataIdT, Variant};

const TRANSFORMATION_NAME_PREFIX: &str = "TRANSFORMATION_";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransformationType {
    Cypher,
    CustomFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transformation {
    pub name: String,
    pub variant: Variant,
    pub export_resources: Vec<(DataIdT, ResourceId)>,
    pub source_field_ids: Vec<ResourceId>,
    pub body: String,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub owners: Vec<String>,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            name: format!("{}{}", TRANSFORMATION_NAME_PREFIX, Utc::now().timestamp()),
            variant: Variant::Default(),
            source_field_ids: Vec::new(),
            export_resources: Vec::new(),
            body: String::new(),
            description: None,
            tags: HashMap::new(),
            owners: Vec::new(),
        }
    }
}

impl ResourceOp for Transformation {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", &self.variant, "Transformation", &self.name)
    }
}

impl Transformation {
    pub fn get_data_id(&self, resource_id: &ResourceId) -> DataIdT {
        self.export_resources
            .iter()
            .find(|(_, rid)| rid == resource_id)
            .unwrap_or_else(|| {
                panic!("resource id should be in export_resources, but got {resource_id}")
            })
            .0
    }
}
