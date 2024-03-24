mod transformation_op;
use log::debug;
use transformation_op::{flatten_handlers, DAGOp};
pub use transformation_op::{
    ExecutionContext, TransformationIOT, TransformationOp, TransformationOutputHandler,
};

mod built_in_op;
pub(crate) use built_in_op::BuiltInOp;

mod aggregate_op;
pub(crate) use aggregate_op::AggregateOp;

#[cfg(test)]
mod tests;

use super::{DataIdT, TransformationData, TransformationError};
use crate::{infra::pi::*, InfraIdentifier, InfraManager, SeResult};
use futures::future::join_all;
use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
    rc::Rc,
};

#[derive(Debug)]
pub struct TransformationPlan {
    ops: HashMap<DataIdT, DAGOp>,
    materializing_data_ids: HashMap<DataIdT, InfraIdentifier>,
    root_op_ids: Vec<DataIdT>,
    sink_infras: HashMap<DataIdT, InfraIdentifier>,
}

impl TransformationPlan {
    #[cfg(test)] // now only for testing use
    fn get_op(&self, id: DataIdT) -> Option<&DAGOp> {
        self.ops.get(&id)
    }

    pub fn construct(
        materializing_data_ids: Vec<DataIdT>,
        data_vec: &Vec<Rc<dyn TransformationData>>,
        sink_infras: &HashMap<DataIdT, InfraIdentifier>,
    ) -> Self {
        let mut ops: HashMap<DataIdT, DAGOp> = HashMap::new();
        materializing_data_ids.iter().for_each(|id| {
            if ops.get(id).is_none() {
                TransformationPlan::traverse_upstreams(*id, data_vec, &mut ops);
                ops.insert(*id, DAGOp::new(&data_vec[*id]));
            }
        });
        let root_op_ids = ops
            .iter()
            .filter_map(|(id, op)| if op.has_upstream() { None } else { Some(*id) })
            .collect();
        TransformationPlan {
            ops,
            materializing_data_ids: materializing_data_ids
                .into_iter()
                .map(|id| (id, sink_infras.get(&id).unwrap().clone()))
                .collect(),
            root_op_ids,
            sink_infras: sink_infras.clone(),
        }
    }

    pub fn orchestrate_infras(&mut self, infra_manager: &InfraManager) {
        debug!("sink infras: {:?}", self.sink_infras);
        for (id, op) in &mut self.ops {
            op.set_connectors(infra_manager, self.sink_infras.get(id).unwrap());
        }
        self.set_output_storage_types();
        self.set_input_storage_types();
    }

    // TODO(tatiana): optimize before execution
    /// TransformationPlan adopts a push-based execution model
    pub async fn execute(
        &self,
        infra_manager: &InfraManager,
    ) -> SeResult<TransformationOutputHandler> {
        let context = ExecutionContext::new(&self.ops, &self.materializing_data_ids, infra_manager);
        let mut streams = Vec::new();
        self.root_op_ids.iter().for_each(|id| {
            streams.push(
                self.ops
                    .get(id)
                    .unwrap()
                    .execute(context.clone(), Vec::new()),
            );
        });
        let mut handlers = Vec::new();
        for result in join_all(streams).await {
            handlers.push(result?);
        }
        // TODO(tatiana): cleanup intermediate outputs.
        Ok(flatten_handlers(handlers))
    }

    fn traverse_upstreams(
        id: DataIdT,
        data_vec: &Vec<Rc<dyn TransformationData>>,
        ops: &mut HashMap<DataIdT, DAGOp>,
    ) {
        #[cfg(debug_assertions)] // avoid stack overflow
        assert!(!data_vec[id].get_parent_data_ids().contains(&id));

        data_vec[id]
            .get_parent_data_ids()
            .iter()
            .for_each(|parent_id| {
                if let Some(parent_op) = ops.get_mut(parent_id) {
                    parent_op.add_downstream(id);
                } else {
                    TransformationPlan::traverse_upstreams(*parent_id, data_vec, ops);
                    let mut parent_op = DAGOp::new(&data_vec[*parent_id]);
                    parent_op.add_downstream(id);
                    ops.insert(*parent_id, parent_op);
                }
            });
    }

    // select input source storage types
    fn set_input_storage_types(&mut self) {
        let input_storages = self
            .ops
            .iter()
            .filter_map(|(id, op)| {
                if !op.has_upstream() {
                    None
                } else {
                    Some((
                        *id,
                        op.get_upstream_ids()
                            .iter()
                            .map(|upstream_id| {
                                let upstream = self.ops.get(upstream_id).unwrap();
                                // if the current upstream output to a sink infra, the op selects a source type that is supported by the infra
                                let source = if upstream.is_transport_to_storage() {
                                    op.get_execution_connector().select_source(
                                        upstream.get_sink_connector().get_supported_sinks(),
                                    )
                                } else if !upstream.has_upstream() {
                                    // if the current upstream is a source op, the op selects a source type that is supported by the source infra
                                    op.get_execution_connector().select_source(
                                        upstream.get_execution_connector().get_supported_sinks(),
                                    )
                                } else {
                                    // otherwise the upstream output type is already determined by this op and its peers
                                    upstream
                                        .get_common_args()
                                        .sink_storage_type()
                                        .cloned()
                                        .unwrap()
                                };
                                source
                            })
                            .collect::<Vec<_>>(),
                    ))
                }
            })
            .collect::<Vec<_>>();
        input_storages.into_iter().for_each(|(id, storages)| {
            self.ops
                .get_mut(&id)
                .unwrap()
                .get_common_args_mut()
                .set_source_storage_type(storages);
        });
    }

    // select the output storage type for each op
    fn set_output_storage_types(&mut self) {
        let output_storages = self
            .ops
            .iter()
            .filter_map(|(id, op)| {
                // if the op's output is in a sink, the output storage type is decided considering the sink infra
                let sinks = if op.is_transport_to_storage() {
                    Some(op.get_sink_connector().get_supported_sources())
                } else if  !op.has_downstream() { // if the op executes and persists the results in the same infra
                    Some(op.get_execution_connector().get_supported_sources())
                } else if !op.has_upstream() {
                    None
                } else  {
                    // otherwise the op's output is to be directly consumed by the downstream op(s), try to select a storage type that suits all
                    let mut intersection = HashMap::<Storage, u32>::new();
                    op.get_downstream_ids().iter().for_each(|downstream_id| {
                        self.ops
                            .get(downstream_id)
                            .unwrap()
                            .get_execution_connector()
                            .get_supported_sources()
                            .iter()
                            .for_each(|storage_type| {
                                intersection
                                    .entry(storage_type.clone())
                                    .or_default()
                                    .add_assign(1);
                            });
                    });
                    let sinks = intersection
                        .iter()
                        .filter_map(|(storage, count)| {
                            if *count as usize == op.get_downstream_ids().len() {
                                Some(storage.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    assert!(
                        !sinks.is_empty(),
                        "Need to insert a sink infra for data transport. op {op:?}, {} downstreams. {intersection:?}", op.get_downstream_ids().len()
                    );
                    Some(sinks)
                }  ;
                sinks.map(|sinks| {
                (*id, op.get_execution_connector().select_sink(sinks))
                })
            })
            .collect::<Vec<_>>();
        output_storages.into_iter().for_each(|(id, storage)| {
            self.ops
                .get_mut(&id)
                .unwrap()
                .get_common_args_mut()
                .set_sink_storage_type(storage);
        });
    }
}

pub(super) fn set_validate_infras(
    infra_manager: &InfraManager,
    data_vec: &[Rc<dyn TransformationData>],
    data_id: DataIdT,
    sink_infras: &HashMap<DataIdT, InfraIdentifier>,
) -> SeResult<()> {
    let data = &data_vec[data_id];
    let execution_infra_id = data
        .get_context()
        .get_infra_id()
        .expect("Now assuming the execution infra of all transformation data are specified");
    // support relational transformation later
    let execution_infra = infra_manager
        .get_graph_transformation_infra(&execution_infra_id)
        .unwrap();
    // check if the execution infra has the required functionality other than GAF::Source
    if data.get_func().ne(&GAF::Source) && !execution_infra.supports_func(&data.get_func()) {
        return Err(Box::new(TransformationError::FuncNotSupported {
            func: data.get_func(),
            infra_id: execution_infra_id,
        }));
    }
    // check if the execution infra can output to the sink infra
    if let Some(sink_infra_id) = sink_infras.get(&data_id) {
        if execution_infra_id.ne(sink_infra_id) {
            let sink_infra = infra_manager.get_storage_infra(sink_infra_id).unwrap();
            if !check_data_transport(
                execution_infra.get_supported_sinks(),
                sink_infra.get_supported_sources(),
            ) {
                return Err(Box::new(TransformationError::DataTransportError {
                    upstream_infra_id: execution_infra_id,
                    downstream_infra_id: sink_infra_id.clone(),
                }));
            }
        }
    }
    for parent in data.get_parent_data_ids() {
        // check if the execution infra can input from parent sink infra
        let parent_sink_infra_id = sink_infras.get(parent).unwrap();
        if execution_infra_id.ne(parent_sink_infra_id) {
            let parent_sink = infra_manager
                .get_storage_infra(parent_sink_infra_id)
                .expect("parent sink should be set before set_validate_infras on the current data");
            if !check_data_transport(
                parent_sink.get_supported_sinks(),
                execution_infra.get_supported_sources(),
            ) {
                return Err(Box::new(TransformationError::DataTransportError {
                    upstream_infra_id: parent_sink_infra_id.clone(),
                    downstream_infra_id: execution_infra_id,
                }));
            }
        }
    }
    Ok(())
}

pub(super) fn check_data_transport(
    upstream_types: Vec<Storage>,
    downstream_types: Vec<Storage>,
) -> bool {
    let supported_types = upstream_types.into_iter().collect::<HashSet<_>>();
    for sink_type in downstream_types {
        if supported_types.contains(&sink_type) {
            return true;
        }
    }
    false
}
