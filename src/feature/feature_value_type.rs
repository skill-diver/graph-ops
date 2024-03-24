use pyo3::{exceptions::PyValueError, prelude::*};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FeatureValueType {
    String,
    Int,
    Float,
    Boolean,
    Date,
    Time,
    DateTime,
    Duration,
    Topology,
    Array(Box<FeatureValueType>),
}

impl FromStr for FeatureValueType {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Array") {
            Ok(FeatureValueType::Array(Box::new(
                FeatureValueType::from_str(&s[6..s.len() - 1])?,
            )))
        } else {
            Ok(serde_json::from_str(&format!("\"{s}\""))?)
        }
    }
}

impl FromPyObject<'_> for FeatureValueType {
    fn extract(ob: &'_ pyo3::PyAny) -> PyResult<Self> {
        let str = ob.extract::<String>()?;
        FeatureValueType::from_str(&str).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

impl IntoPy<PyObject> for FeatureValueType {
    fn into_py(self, py: Python<'_>) -> PyObject {
        format!("{self:?}").into_py(py)
    }
}

#[test]
fn test_feature_type_from_str() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", FeatureValueType::from_str("Array(Array(Float))")?);
    assert!(
        FeatureValueType::from_str("Array(Array(Float))")?
            == FeatureValueType::Array(Box::new(FeatureValueType::Array(Box::new(
                FeatureValueType::Float
            ))))
    );
    FeatureValueType::from_str("List").unwrap_err();
    Ok(())
}
