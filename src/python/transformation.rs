mod argparse;
use argparse::*;

use pyo3::{exceptions::*, prelude::*};
use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

use crate::{
    feature::ResourceId, infra::pi::GAF, transformation::DataFrameBase, transformation::*, Entity,
    Field, InfraIdentifier, Topology,
};

use super::ClientInner;

#[pyclass(unsendable, module = "ofnil", name = "PipelineContext")]
pub(crate) struct PyPipelineContext {
    pub(crate) client: Py<ClientInner>,
    pub(crate) inner: Rc<RefCell<TransformationContext>>,
}

#[pymethods]
impl PyPipelineContext {
    #[new]
    fn new(client: Py<ClientInner>) -> Self {
        Self {
            client,
            inner: TransformationContext::new(),
        }
    }

    fn finalize(
        self_: PyRef<Self>,
        entities: Option<Vec<PyObject>>,
        fields: Option<Vec<Field>>,
        topos: Option<Vec<Topology>>,
    ) -> PyResult<ResourceId> {
        let py = self_.py();
        let entities = entities.map(|vec| {
            vec.iter()
                .map(|obj| obj.extract::<Entity>(py).unwrap())
                .collect::<Vec<_>>()
        });
        self_
            .client
            .borrow(py)
            .rt
            .block_on(finalize_transformation(
                &self_.client.borrow(py).fs,
                &self_.inner,
                if let Some(entities) = &entities {
                    entities.iter().collect()
                } else {
                    Vec::new()
                },
                if let Some(fields) = &fields {
                    fields.iter().collect()
                } else {
                    Vec::new()
                },
                if let Some(topos) = &topos {
                    topos.iter().collect()
                } else {
                    Vec::new()
                },
            ))
            .map_err(|e| {
                PyRuntimeError::new_err(format!("Error in PyPipelineContext.finalize(). {e}"))
            })
    }
}

#[pyclass(unsendable, module = "ofnil", name = "GraphFrame")]
pub(crate) struct PyGraphFrame {
    pub(crate) inner: Rc<dyn GraphComputationOps>,
}

#[pymethods]
impl PyGraphFrame {
    pub(crate) fn apply_procedure(
        self_: PyRef<Self>,
        context: PyRef<crate::python::PyPipelineContext>,
        procedure_name: String,
        args: Option<HashMap<String, PyObject>>,
    ) -> PyResult<PyObject> {
        let py = self_.py();
        let gaf = GAF::from_str(&procedure_name.to_lowercase()).map_err(|e| {
            PyValueError::new_err(format!(
                "apply_procedure({procedure_name}, {args:?}). \"{procedure_name}\" is not a valid GraphAnalyticFunc. {e}"
            ))
        })?;
        let common_args = match args.as_ref() {
            Some(args) => Some(parse_common_args(args, py)?),
            None => None,
        };
        match gaf {
            GAF::PageRank => {
                let args = args.as_ref().expect("page_rank requires args");
                let (entities, target_node_entity) =
                    parse_graph_algorithm_args(&procedure_name, args, py, context)?;
                self_
                    .inner
                    .page_rank(
                        entities,
                        target_node_entity,
                        parse_optional_args(args, "damping_factor", py)?,
                        parse_optional_args(args, "max_iteration",py)?,
                        parse_optional_args(args, "tolerance",py)?,
                        common_args,
                    )
                    .map(|df| {PyDataFrame::new(df).into_py(py)})
            }
            GAF::TriangleCount => {
                let args = args.as_ref().expect("triangle_count requires args");
                let (entities, target_node_entity) =
                    parse_graph_algorithm_args(&procedure_name, args, py, context)?;
                self_
                    .inner
                    .triangle_count(entities, target_node_entity, common_args)
                    .map(|df| PyDataFrame::new(df).into_py(py))
            }
            GAF::AggregateNeighbors => {
                let args = args.as_ref().expect("aggregate_neighbors requires args");
                let rt = &context.client.borrow(py).rt;
                let edge_entity_id = parse_optional_args::<ResourceId>(args, "edge_entity", py)?;
                let target_node_entity_id = parse_args::<ResourceId>(args, "target_node_entity", py)?;
                let edge_entity = if let Some(id) = edge_entity_id {
                   Some(rt.block_on(
                    context.client.borrow(py).fs.registry.get_entity(&id))
                        .map_err(|e| { PyValueError::new_err(format!(
                            "Error apply_procedure({procedure_name}, {args:?}). Cannot get specified entity from registry. {e}"
                            ))
                        })? )
                } else {
                    None
                };
                let target_node_entity = rt.block_on(
                    context.client.borrow(py).fs.registry.get_entity(&target_node_entity_id))
                        .map_err(|e| { PyValueError::new_err(format!(
                            "Error apply_procedure({procedure_name}, {args:?}). Cannot get specified entity from registry. {e}"
                            ))
                        })?;
                self_
                    .inner
                    .aggregate_neighbors(
                        edge_entity,
                        target_node_entity,
                        parse_args(args, "properties", py)?,
                        parse_args(args, "aggregator", py)?,
                        common_args,
                    )
                    .map(|df| PyDataFrame::new(df).into_py(py))
            }
            _ => panic!("Unsupported procedure: {procedure_name}"),
        }
        .map_err(|e| {
            PyValueError::new_err(format!(
                "Error apply_procedure({procedure_name}, {args:?}). {e}"
            ))
        })
    }
}

#[pyclass(unsendable, module = "ofnil", name = "DataFrame")]
pub(crate) struct PyDataFrame {
    pub(crate) inner: Rc<dyn DataFrameBase>,
}

impl PyDataFrame {
    pub(crate) fn new(pointer: Rc<dyn DataFrameBase>) -> Self {
        PyDataFrame { inner: pointer }
    }
}

#[pymethods]
impl PyDataFrame {
    pub(crate) fn apply_procedure(
        self_: PyRef<Self>,
        _context: PyRef<crate::python::PyPipelineContext>,
        procedure_name: String,
        args: HashMap<String, PyObject>,
    ) -> PyResult<PyObject> {
        Ok(match procedure_name.to_lowercase().as_str() {
            "select" => {
                PyDataFrame::new(self_.inner.select(parse_args(&args, "exprs", self_.py())?))
                    .into_py(self_.py())
            }
            _ => panic!("Unknown procedure name {procedure_name}"),
        })
    }

    pub(crate) fn export(
        self_: PyRef<Self>,
        sink_infra_id: InfraIdentifier,
    ) -> PyResult<Vec<Field>> {
        Ok(self_.inner.export(&sink_infra_id))
    }
}
