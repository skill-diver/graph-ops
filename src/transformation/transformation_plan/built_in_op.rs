use super::{TransformationIOT, TransformationOp};
use crate::{
    infra::pi::{TransformationConnector, GAF},
    transformation::{
        CommonTransformationArgs, DataIdT, TransformationArgs, TransformationOutputHandler,
    },
    InfraManager, SeResult,
};
use log::info;

/// This operation simply calls a built-in function of the underlying connector
#[derive(Debug)]
pub(crate) struct BuiltInOp {
    args: TransformationArgs,
    common_args: CommonTransformationArgs,
    /// The connector to execute the built-in function
    execution_connector: Option<Box<dyn TransformationConnector>>, // required for execution
    func: GAF,
}

impl BuiltInOp {
    /// # Parameters
    /// func: The built-in function to run.
    /// args: The arguments for the built-in function.
    /// common_args: The configurations for general execution (infra and storage types, etc.).
    pub(crate) fn new(
        func: GAF,
        args: TransformationArgs,
        common_args: CommonTransformationArgs,
    ) -> Self {
        BuiltInOp {
            func,
            args,
            common_args,
            execution_connector: None,
        }
    }

    pub(super) fn get_args(&self) -> &TransformationArgs {
        &self.args
    }
}

#[async_trait::async_trait(?Send)]
impl TransformationOp for BuiltInOp {
    async fn execute(
        &self,
        data_id: DataIdT,
        input: &TransformationIOT,
    ) -> SeResult<TransformationOutputHandler> {
        info!(
            "Executing {:?} for data_id: {}, args {:#?}",
            self.func, data_id, self.args
        );
        let executor = self.get_execution_connector().get_graph_executor(
            &self.func,
            self.args.clone(),
            self.common_args.source_storage_types().clone(),
            self.common_args.sink_storage_type().cloned().unwrap(),
        );
        Ok(executor.execute(input).await.unwrap())
    }

    fn get_common_args(&self) -> &CommonTransformationArgs {
        &self.common_args
    }

    fn get_common_args_mut(&mut self) -> &mut CommonTransformationArgs {
        &mut self.common_args
    }

    fn set_execution_connector(&mut self, infra_manager: &InfraManager) {
        self.execution_connector =
            Some(infra_manager.get_graph_transformation_infra_cloned(
                self.common_args.infra_id().as_ref().unwrap(),
            ))
            .unwrap()
    }

    fn get_execution_connector(&self) -> &dyn TransformationConnector {
        self.execution_connector.as_ref().unwrap().as_ref()
    }
}
