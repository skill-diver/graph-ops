use pyo3::{exceptions::*, prelude::*};
use std::collections::HashMap;

use crate::{
    feature::ResourceId, transformation::CommonTransformationArgs, Entity, InfraIdentifier,
};

pub(super) fn parse_common_args(
    args: &HashMap<String, PyObject>,
    py: Python<'_>,
) -> PyResult<CommonTransformationArgs> {
    Ok(CommonTransformationArgs::new(parse_optional_args::<
        InfraIdentifier,
    >(args, "infra", py)?))
}

pub(super) fn parse_graph_algorithm_args(
    procedure_name: &str,
    args: &HashMap<String, PyObject>,
    py: Python<'_>,
    context: PyRef<crate::python::PyPipelineContext>,
) -> PyResult<(Vec<Entity>, Entity)> {
    let registry = &context.client.borrow(py).fs.registry;
    let rt = &context.client.borrow(py).rt;

    let entities = parse_args::<Vec<ResourceId>>(args, "entities", py)?;
    let target_node_entity = parse_args::<ResourceId>(args, "target_node_entity", py)?;
    rt.block_on(async {
                match registry.get_entities(entities.iter().collect()).await {
                    Ok(entities) => {
                        registry.get_entity(&target_node_entity).await.map(|target|{
                            (entities, target)
                        })
                    },
                    Err(e) => Err(e),
                }
            }).map_err(|e| { PyValueError::new_err(format!(
                            "Error apply_procedure({procedure_name}, {args:?}). Cannot get specified entity from registry. {e}"
                        ))
                    })
}

pub(super) fn parse_args<'p, T>(
    args: &'p HashMap<String, PyObject>,
    key: &str,
    py: Python<'p>,
) -> PyResult<T>
where
    T: FromPyObject<'p>,
{
    args.get(key)
        .ok_or(PyKeyError::new_err(format!(
            "expected key \"{key}\" in args"
        )))?
        .extract(py)
        .map_err(|e| PyTypeError::new_err(format!("Error in extracting {key}. {e}")))
}

pub(super) fn parse_optional_args<'p, T>(
    args: &'p HashMap<String, PyObject>,
    key: &str,
    py: Python<'p>,
) -> PyResult<Option<T>>
where
    T: FromPyObject<'p>,
{
    if let Some(obj) = args.get(key) {
        obj.extract(py)
            .map_err(|e| PyTypeError::new_err(format!("Error in extracting {key}. {e}")))
    } else {
        Ok(None)
    }
}
