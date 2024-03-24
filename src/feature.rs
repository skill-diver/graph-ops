//! Defines resources in registry. The main resources include [Entity], [Field],
//! [Topology], [Graph], and [FeatureView].
//!
//! See <https://github.com/ofnil/ofnil/blob/main/docs/concepts.md> for detailed explanation
//! on Ofnil feature registry.

mod entity;
mod feature_value_type;
mod feature_view;
mod field;
mod graph;
mod topology;
mod transformation;
mod variant;

use std::fmt::Debug;

use pyo3::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use entity::{EdgeEntity, Entity, VertexEntity};
pub use feature_value_type::FeatureValueType;
pub use feature_view::{TableFeatureView, TopologyFeatureView};
pub use field::Field;
pub use graph::Graph;
pub use topology::{Topology, TopologyType};
pub use transformation::{Transformation, TransformationType};
pub use variant::Variant;
pub type ResourceId = String;

#[cfg(feature = "serving")]
use crate::serving::GraphDatasetRenderingOptions;
use crate::InfraIdentifier;

pub trait ResourceOp: Serialize + DeserializeOwned + Debug + Clone {
    fn resource_id(&self) -> ResourceId;
    fn id_to_name(id: &str) -> &str {
        id.rsplit_once('/').unwrap().1
    }

    fn transformation_id(&self) -> Option<ResourceId> {
        None
    }

    fn sink_infra_id(&self) -> Option<InfraIdentifier> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GraphDataset {
    pub name: String,
    pub variant: Variant,
    pub description: Option<String>,
    pub table_feature_views: Vec<TableFeatureView>,
    pub topology_feature_views: Vec<TopologyFeatureView>,
    #[cfg(feature = "serving")]
    pub rendering_opt: GraphDatasetRenderingOptions,
    pub deployed: bool,
}

impl GraphDataset {
    pub fn new(
        name: &str,
        table_feature_views: Vec<TableFeatureView>,
        topology_feature_views: Vec<TopologyFeatureView>,
        #[cfg(feature = "serving")] rendering_opt: GraphDatasetRenderingOptions,
    ) -> Self {
        Self {
            name: name.to_string(),
            table_feature_views,
            topology_feature_views,
            #[cfg(feature = "serving")]
            rendering_opt,
            ..Default::default()
        }
    }
}

impl ResourceOp for GraphDataset {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", &self.variant, "GraphDataset", &self.name)
    }
}

pub(crate) fn init_module(module: &PyModule) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(entity::vertex_entity, module)?)?;
    module.add_function(wrap_pyfunction!(entity::edge_entity, module)?)?;
    module.add_function(wrap_pyfunction!(field::fields, module)?)?;
    module.add_class::<Graph>()?;
    Ok(())
}
