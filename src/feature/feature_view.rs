use super::{ResourceId, ResourceOp, TopologyType};
#[cfg(feature = "serving")]
use crate::serving::{
    FeatureRenderingOptions, FeatureServingOutputType, ServingMode, TopologyRenderingOptions,
    TopologyServingLayout,
};
use crate::{Field, Topology, Variant};
use chrono::{serde::ts_seconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableFeatureView {
    pub name: String,
    pub variant: Variant,
    pub entity_id: ResourceId,      // entity resource id
    pub field_ids: Vec<ResourceId>, // field resource id
    pub online: bool,
    pub description: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: HashMap<String, String>,
    pub owner: Option<String>,
    #[cfg(feature = "serving")]
    pub rendering_opt: FeatureRenderingOptions,
}

impl ResourceOp for TableFeatureView {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", &self.variant, "TableFeatureView", &self.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyFeatureView {
    pub name: String,
    pub variant: Variant,
    pub topology_type: TopologyType,
    pub online: bool,
    pub topology_ids: Vec<ResourceId>, // topology resource id
    pub description: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
    pub tags: HashMap<String, String>,
    pub owners: Vec<String>,
    #[cfg(feature = "serving")]
    pub rendering_opt: TopologyRenderingOptions,
}

impl ResourceOp for TopologyFeatureView {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", self.variant, "TopologyFeatureView", &self.name)
    }
}

impl TableFeatureView {
    pub fn default(name: &str, entity_id: ResourceId, fields: &[Field]) -> Self {
        TableFeatureView {
            name: name.to_string(),
            variant: Variant::Default(),
            entity_id,
            field_ids: fields.iter().map(|f| f.resource_id()).collect(),
            created_at: Some(Utc::now()),
            online: false,
            description: None,
            updated_at: None,
            tags: HashMap::new(),
            owner: None,
            #[cfg(feature = "serving")]
            rendering_opt: FeatureRenderingOptions::new(
                FeatureServingOutputType::NdArray,
                ServingMode::PythonBinding,
            ),
        }
    }
}

impl TopologyFeatureView {
    pub fn default(name: &str, topos: &[Topology]) -> Self {
        Self {
            name: name.to_string(),
            variant: Variant::Default(),
            topology_type: TopologyType::AdjacencyMatrix,
            online: false,
            topology_ids: topos.iter().map(|g| g.resource_id()).collect(),
            description: None,
            created_at: Some(Utc::now()),
            updated_at: None,
            tags: HashMap::new(),
            owners: Vec::new(),
            #[cfg(feature = "serving")]
            rendering_opt: TopologyRenderingOptions::new(
                TopologyServingLayout::CompressedSparseRow,
                ServingMode::PythonBinding,
            ),
        }
    }
}
