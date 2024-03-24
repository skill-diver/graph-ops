//! Specifies transformation workflow and generates execution plan.

pub mod built_in_fns;
pub mod dataframes;
pub mod graph;
pub mod transformation_args;
pub mod utils;

mod cypher_result;
mod result_handler;
mod schema;
mod transformation_context;
mod transformation_plan;

use log::info;
use std::{cell::RefCell, error::Error, rc::Rc};

// re-export for public interface
pub use built_in_fns::BuiltInFnArgs;
pub use dataframes::DataFrameBase;
pub use graph::{GraphBase, GraphComputationOps};
pub use transformation_args::{CommonTransformationArgs, GraphProjectionArgs, TransformationArgs};
pub use transformation_context::TransformationContext;
pub use transformation_plan::{TransformationIOT, TransformationOutputHandler};

// re-export crate-level commonly used items
pub(crate) use schema::*;
pub(crate) use usize as DataIdT;

use built_in_fns::*;
use dataframes::DataFrame;
use graph::SingleGraph;
#[allow(unused)] // TODO(tatiana): to be finished
use graph::{EdgeSelectGraph, Selector, VertexSelectGraph};
use transformation_context::DataTransformationContext;
use transformation_plan::{TransformationOp, TransformationPlan};

use crate::{
    feature::ResourceId, feature::ResourceOp, infra::pi::GraphAnalyticFunc, FeatureStore,
    InfraIdentifier, Variant,
};

/// A TransformationData instance is registered in the TransformationContext, and its
/// implementation has a DataTransformationContext.
#[typetag::serde]
pub trait TransformationData {
    // context getter to hide direct member access to enable impl trait function on multiple structs
    fn get_context(&self) -> &DataTransformationContext;

    fn get_producer_op(&self) -> Box<dyn TransformationOp>;

    fn get_func(&self) -> GraphAnalyticFunc;

    fn get_data_id(&self) -> DataIdT {
        self.get_context().id
    }

    fn get_parent_data_ids(&self) -> &Vec<DataIdT> {
        &self.get_context().parent_data_ids
    }
}

impl std::fmt::Debug for dyn TransformationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("data id {}", self.get_data_id()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransformationError {
    #[error(
        "Data transport from {upstream_infra_id:?} to {downstream_infra_id:?} is unsupported."
    )]
    DataTransportError {
        upstream_infra_id: InfraIdentifier,
        downstream_infra_id: InfraIdentifier,
    },
    #[error("Infra {infra_id:?} does not support {func:?}.")]
    FuncNotSupported {
        infra_id: InfraIdentifier,
        func: GraphAnalyticFunc,
    },
}

// TODO(tatiana): test infra validation
/// Validate the transformation and then register it in the feature registry.
///
/// The transformation specifies a dataflow and the corresponding execution and sink infra.
/// To finalize a transformation, we need to validate it by three requirements:
/// 1. For each non-source [TransformationData] in the dataflow, the infra used to compute
///    the data must support the transformation functionality in demand.
/// 2. For each non-source `TransformationData`, the execution infra must be able to output
///    to the sink infra if they are not the same.
/// 3. For each non-source `TransformationData`, the execution infra must be able to input
///    from the parent data sinks if they are not the same.
///
/// <p style="color:red">TODO: The logic to decide execution and sink infra for a `TransformationData` when unspecified
/// is to be discussed.</p>
pub async fn finalize_transformation(
    fs: &FeatureStore,
    tc: &Rc<RefCell<TransformationContext>>,
    entities: Vec<&impl ResourceOp>,
    fields: Vec<&impl ResourceOp>,
    topos: Vec<&impl ResourceOp>,
) -> Result<ResourceId, Box<dyn Error>> {
    tc.as_ref()
        .borrow_mut()
        .set_and_validate_infras(fs.infra_manager())?;
    let transformation = tc
        .as_ref()
        .borrow_mut()
        .build_transformation(None, Variant::default())?
        .unwrap()
        .to_owned();
    info!("finalized transformation: {:?}", transformation);
    info!("transformation body: {}", transformation.body);
    fs.registry.register_resource(&transformation).await?;
    fs.registry.register_resources(&entities).await?;
    fs.registry.register_resources(&fields).await?;
    fs.registry.register_resources(&topos).await?;
    Ok(transformation.resource_id())
}
