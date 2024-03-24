use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Variant {
    Default(),
    UserDefined(String),
}

impl Default for Variant {
    fn default() -> Variant {
        Self::Default()
    }
}

impl Variant {
    pub fn user_defined<T: AsRef<str>>(version: T) -> Variant {
        Self::UserDefined(version.as_ref().to_string())
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let version = match self {
            Variant::Default() => "default",
            Variant::UserDefined(version) => version.as_str(),
        };
        write!(f, "{version}")
    }
}

impl IntoPy<PyObject> for Variant {
    fn into_py(self, py: Python<'_>) -> PyObject {
        format!("{self}").into_py(py)
    }
}
