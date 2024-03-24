//! Feature serving specifications for on-demand feature processing and rendering.

pub mod output_type;
pub mod sampling_conf;
pub mod serving_mode;

pub use output_type::{FeatureServingOutputType, TopologyServingLayout};
pub use sampling_conf::{BfsConf, NeighborSamplingConf, SamplingConf};
pub use serving_mode::ServingMode;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// For TableFeatureView.
#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeatureRenderingOptions {
    #[pyo3(get, set)]
    output_type: FeatureServingOutputType,
    #[pyo3(get, set)]
    mode: ServingMode,
}

/// For TopologyFeatureView.
#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyRenderingOptions {
    #[pyo3(get, set)]
    layout: TopologyServingLayout,
    #[pyo3(get, set)]
    mode: ServingMode,
}

/// For GraphDataset.
#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GraphDatasetRenderingOptions {
    #[pyo3(get, set)]
    pub sampling: Option<SamplingConf>,
}

#[pymethods]
impl GraphDatasetRenderingOptions {
    #[new]
    #[pyo3(signature = (fanouts, seed_type=None, edge_types=None, replace=false))]
    pub fn sample_k_hop_neighbors(
        fanouts: Vec<u32>,
        seed_type: Option<String>,
        edge_types: Option<Vec<String>>,
        replace: Option<bool>,
    ) -> Self {
        Self {
            sampling: Some(SamplingConf::NeighborSampling(NeighborSamplingConf::new(
                fanouts, seed_type, edge_types, replace,
            ))),
        }
    }
}

impl FeatureRenderingOptions {
    pub fn new(output_type: FeatureServingOutputType, mode: ServingMode) -> Self {
        Self { output_type, mode }
    }
}

#[pymethods]
impl FeatureRenderingOptions {
    #[new]
    pub fn default(output_type: FeatureServingOutputType) -> Self {
        Self {
            output_type,
            mode: ServingMode::PythonBinding,
        }
    }
}

impl TopologyRenderingOptions {
    pub fn new(layout: TopologyServingLayout, mode: ServingMode) -> Self {
        Self { layout, mode }
    }
}

#[pymethods]
impl TopologyRenderingOptions {
    #[new]
    pub fn default(layout: TopologyServingLayout) -> Self {
        Self {
            layout,
            mode: ServingMode::PythonBinding,
        }
    }
}

pub(crate) fn init_module(module: &PyModule) -> PyResult<()> {
    module.add_class::<FeatureRenderingOptions>()?;
    module.add_class::<GraphDatasetRenderingOptions>()?;
    module.add_class::<output_type::FeatureServingOutputType>()?;
    module.add_class::<output_type::TopologyServingLayout>()?;
    module.add_class::<sampling_conf::BfsConf>()?;
    module.add_class::<sampling_conf::NeighborSamplingConf>()?;
    Ok(())
}
