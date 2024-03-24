use pyo3::prelude::pyclass;
use serde::{Deserialize, Serialize};

#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FeatureServingOutputType {
    NdArray,
}

#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TopologyServingLayout {
    CompressedSparseRow,
}
