use pyo3::{
    prelude::{pyclass, pymethods},
    FromPyObject, IntoPy, PyObject,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SamplingConf {
    Bfs(BfsConf),
    NeighborSampling(NeighborSamplingConf),
}

impl IntoPy<PyObject> for SamplingConf {
    fn into_py(self, py: pyo3::Python<'_>) -> PyObject {
        match self {
            SamplingConf::Bfs(conf) => conf.into_py(py),
            SamplingConf::NeighborSampling(conf) => conf.into_py(py),
        }
    }
}

impl FromPyObject<'_> for SamplingConf {
    fn extract(ob: &'_ pyo3::PyAny) -> pyo3::PyResult<Self> {
        let try_bfs: Result<BfsConf, _> = ob.extract();
        if let Ok(bfs) = try_bfs {
            Ok(SamplingConf::Bfs(bfs))
        } else {
            Ok(SamplingConf::NeighborSampling(ob.extract()?))
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BfsConf {}

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeighborSamplingConf {
    pub seed_type: Option<String>,
    pub fanouts: Vec<u32>,
    pub edge_types: Option<Vec<String>>,
    pub replace: bool,
}

#[pymethods]
impl NeighborSamplingConf {
    #[new]
    pub fn new(
        fanouts: Vec<u32>,
        seed_type: Option<String>,
        edge_types: Option<Vec<String>>,
        replace: Option<bool>,
    ) -> NeighborSamplingConf {
        if let Some(metapath) = &edge_types {
            assert!(metapath.len() == fanouts.len());
        }
        NeighborSamplingConf {
            seed_type,
            fanouts,
            edge_types,
            replace: replace.unwrap_or(false),
        }
    }
}
