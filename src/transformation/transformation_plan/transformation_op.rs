use crate::{
    infra::pi::{
        storage::{Row, Source, Storage},
        StorageConnector, TransformationConnector,
    },
    transformation::{
        utils::{get_type_of, transport_source_to_sink},
        CommonTransformationArgs, DataIdT, TransformationData,
    },
    InfraIdentifier, InfraManager, SeResult,
};

use async_recursion::async_recursion;
use futures::future::join_all;
use log::info;
use std::{collections::HashMap, rc::Rc, sync::Arc};
use tokio::sync::Mutex;

pub type TransformationIOT = Vec<TransformationOutputHandler>;

// TransformationOutputHandler should be cheap to clone and invariant on clone
#[derive(Clone, Debug)]
pub enum TransformationOutputHandler {
    CypherOutputHandler { data_id: DataIdT, path: String },
    InfraHandler { infra_id: InfraIdentifier },
    TabularSource(Arc<dyn Source<Row>>),
    EmptyOutput,
    // recursion should be only one level
    PropagatedHandlers(TransformationIOT),
}

pub(super) fn flatten_handlers(
    handlers: Vec<TransformationOutputHandler>,
) -> TransformationOutputHandler {
    TransformationOutputHandler::PropagatedHandlers(
        handlers
            .into_iter()
            .flat_map(|result| match result {
                TransformationOutputHandler::PropagatedHandlers(list) => list,
                other => vec![other],
            })
            .collect(),
    )
}

#[async_trait::async_trait(?Send)]
pub trait TransformationOp: std::fmt::Debug {
    async fn execute(
        &self,
        _data_id: DataIdT,
        _input: &TransformationIOT,
    ) -> SeResult<TransformationOutputHandler> {
        unimplemented!("{self:?}")
    }

    fn get_common_args(&self) -> &CommonTransformationArgs {
        unimplemented!("{self:?}")
    }

    fn get_common_args_mut(&mut self) -> &mut CommonTransformationArgs {
        unimplemented!("{self:?}")
    }

    fn set_execution_connector(&mut self, _infra_manager: &InfraManager) {
        unimplemented!("{self:?}")
    }

    fn get_output_connector(&self) -> &dyn StorageConnector {
        unimplemented!("{self:?}")
    }

    fn get_execution_connector(&self) -> &dyn TransformationConnector {
        unimplemented!("{self:?}")
    }
}

#[derive(Debug)]
pub(super) struct DAGOp {
    data_id: DataIdT,          // data id of transformation output
    upstreams: Vec<DataIdT>,   // upstream ops identified by data id
    downstreams: Vec<DataIdT>, // downstream ops identified by data id
    inner_op: Box<dyn TransformationOp>,
    sink_infra: Option<Box<dyn StorageConnector>>,
    sink_infra_id: Option<InfraIdentifier>,
}

impl DAGOp {
    pub(super) fn new(data: &Rc<dyn TransformationData>) -> Self {
        DAGOp {
            data_id: data.get_context().id,
            upstreams: data.get_parent_data_ids().clone(),
            downstreams: Vec::new(),
            inner_op: data.get_producer_op(),
            sink_infra: None,
            sink_infra_id: None,
        }
    }

    fn to_materialize(&self, context: &ExecutionContext) -> bool {
        context.inner.materializing_ids.contains_key(&self.data_id)
    }

    pub(super) fn set_connectors(
        &mut self,
        infra_manager: &InfraManager,
        sink_infra_id: &InfraIdentifier,
    ) {
        // if the execution and sink storage infras are not the same, init the storage connector for output to sink
        if self.get_execution_infra_id().ne(sink_infra_id) {
            self.sink_infra_id = Some(sink_infra_id.clone());
            self.sink_infra = infra_manager.get_storage_infra_cloned(sink_infra_id);
        }
        // setup the transformation connector
        self.inner_op.set_execution_connector(infra_manager);
    }

    pub(super) fn get_sink_connector(&self) -> &dyn StorageConnector {
        self.sink_infra
            .as_ref()
            .expect("some storage connector after set_connectors is called")
            .as_ref()
    }

    pub(super) fn get_execution_connector(&self) -> &dyn TransformationConnector {
        self.inner_op.get_execution_connector()
    }

    pub(super) fn has_downstream(&self) -> bool {
        !self.get_downstream_ids().is_empty()
    }

    pub(super) fn has_upstream(&self) -> bool {
        !self.get_upstream_ids().is_empty()
    }

    pub(super) fn get_downstream_ids(&self) -> &Vec<DataIdT> {
        &self.downstreams
    }

    pub(super) fn get_upstream_ids(&self) -> &Vec<DataIdT> {
        &self.upstreams
    }

    pub(super) fn add_downstream(&mut self, data_id: DataIdT) {
        self.downstreams.push(data_id);
    }

    /// Consume the input, push output to downstream ops for asynchronous execution, and propagate result to materialize through context if applicable.
    /// @returns Results to be materialized along the execution of this op and its downstreams.
    #[async_recursion(?Send)]
    pub async fn execute(
        &self,
        context: ExecutionContext<'async_recursion>,
        input: TransformationIOT,
    ) -> SeResult<TransformationOutputHandler> {
        let input = self.prepare_input(input, &context);
        let output = self.inner_op.execute(self.data_id, &input).await?;
        let output = self.transport_to_storage(output).await?;
        Ok(if self.has_downstream() {
            // output to downstream. merge current output and downstream outputs if this output is to be materialized
            let downstream_outputs = context.out(&self.downstreams, output.clone()).await?;
            if self.to_materialize(&context) {
                flatten_handlers(vec![downstream_outputs, output])
            } else {
                downstream_outputs
            }
        } else {
            assert!(self.to_materialize(&context));
            output
        })
    }

    #[inline]
    pub(super) fn is_transport_to_storage(&self) -> bool {
        self.sink_infra_id.is_some()
    }

    #[inline]
    fn get_execution_infra_id(&self) -> &InfraIdentifier {
        self.inner_op.get_common_args().infra_id().unwrap()
    }

    #[inline]
    fn get_sink_storage_type(&self) -> &Storage {
        self.inner_op
            .get_common_args()
            .sink_storage_type()
            .unwrap_or_else(|| panic!("sink_storage_type is not set. {self:?}"))
    }

    fn prepare_input(
        &self,
        inputs: TransformationIOT,
        context: &ExecutionContext<'_>,
    ) -> TransformationIOT {
        inputs
            .into_iter()
            .map(|input| {
                match &input {
                    TransformationOutputHandler::InfraHandler { infra_id } => {
                        // when the input is a infra handler, get source if the input infra is not the same as the execution infra.
                        if self.get_execution_infra_id().ne(infra_id) {
                            let source = context
                                .infra_manager
                                .get_storage_infra(infra_id)
                                .unwrap()
                                .produce_rows();
                            TransformationOutputHandler::TabularSource(Arc::from(source))
                        } else {
                            input
                        }
                    }
                    TransformationOutputHandler::TabularSource(_)
                    | TransformationOutputHandler::EmptyOutput => input,
                    _ => panic!("unexpected input {input:?}"),
                }
            })
            .collect()
    }

    async fn transport_to_storage(
        &self,
        data: TransformationOutputHandler,
    ) -> SeResult<TransformationOutputHandler> {
        // TODO(tatiana): support other storage types
        if self.is_transport_to_storage() {
            assert!(Storage::OfnilRow.eq(self.get_sink_storage_type()));
            let sink_infra = self.sink_infra.as_ref().unwrap();
            let source = match data {
                TransformationOutputHandler::TabularSource(source) => source,
                TransformationOutputHandler::EmptyOutput => return Ok(data),
                _ => panic!(
                    "Expect Source of {:?}, but got {data:?}",
                    self.get_sink_storage_type()
                ),
            };
            info!(
                "{}-{} output from {:?} to sink {:?}",
                self.data_id,
                get_type_of(&self.inner_op),
                self.get_execution_infra_id(),
                self.sink_infra_id
            );
            let sink = sink_infra.insert_rows(source.get_schema().clone());
            transport_source_to_sink(source.as_ref(), sink.as_ref()).await?;
            Ok(TransformationOutputHandler::InfraHandler {
                infra_id: self.sink_infra_id.clone().unwrap(),
            })
        } else {
            Ok(data)
        }
    }

    pub(super) fn get_common_args(&self) -> &CommonTransformationArgs {
        self.inner_op.get_common_args()
    }

    pub(super) fn get_common_args_mut(&mut self) -> &mut CommonTransformationArgs {
        self.inner_op.get_common_args_mut()
    }
}

/// A simple wrapper of Arc<ExecutionContextImpl>
#[derive(Clone)]
pub struct ExecutionContext<'a> {
    inner: Arc<ExecutionContextImpl<'a>>,
    infra_manager: &'a InfraManager,
}

pub struct ExecutionContextImpl<'a> {
    materializing_ids: &'a HashMap<DataIdT, InfraIdentifier>,
    ops: &'a HashMap<DataIdT, DAGOp>,
    // TODO(tatiana): no need to lock?
    inputs: Mutex<HashMap<DataIdT, TransformationIOT>>, // op id, [(upstream op id, upstream output)]
}

impl<'a> ExecutionContextImpl<'a> {
    pub(super) fn new(
        ops: &'a HashMap<DataIdT, DAGOp>,
        materializing_ids: &'a HashMap<DataIdT, InfraIdentifier>,
    ) -> Arc<Self> {
        Arc::new(ExecutionContextImpl {
            materializing_ids,
            ops,
            inputs: Mutex::new(HashMap::new()),
        })
    }
}

impl<'a> ExecutionContext<'a> {
    pub async fn out(
        &self,
        downstreams: &Vec<DataIdT>,
        output: TransformationOutputHandler,
    ) -> SeResult<TransformationOutputHandler> {
        let mut downstream_ready: Vec<(&DAGOp, TransformationIOT)> = Vec::new();
        // add input to downstreams and get ready downstreams
        // TODO(tatiana): consider the case of streaming op where execution is triggered by output of any upstream instead of blocked to wait for all upstreams
        for downstream in downstreams {
            let mut inputs = self.inner.inputs.lock().await;
            let downstream_input = inputs
                .entry(*downstream)
                .and_modify(|input| input.push(output.clone()))
                .or_insert_with(|| vec![output.clone()]);
            let downstream_op = self.inner.ops.get(downstream).unwrap();
            // the last upstream is finished, no more update to inputs.entry(downstream)
            if downstream_input.len() == downstream_op.get_upstream_ids().len() {
                downstream_ready.push((downstream_op, downstream_input.clone()));
            }
        }
        // execute ready downstreams
        let mut downstream_futures = Vec::new();
        for (downstream_op, downstream_input) in downstream_ready {
            // scheduling goes here if needed
            downstream_futures.push(downstream_op.execute(self.clone(), downstream_input));
        }
        let mut handlers = Vec::new();
        for result in join_all(downstream_futures).await {
            handlers.push(result?);
        }
        Ok(flatten_handlers(handlers))
    }
}

impl<'a> ExecutionContext<'a> {
    pub(super) fn new(
        ops: &'a HashMap<DataIdT, DAGOp>,
        materializing_ids: &'a HashMap<DataIdT, InfraIdentifier>,
        infra_manager: &'a InfraManager,
    ) -> Self {
        Self {
            inner: ExecutionContextImpl::new(ops, materializing_ids),
            infra_manager,
        }
    }
}
