use super::{
    transformation_plan::set_validate_infras, CommonTransformationArgs, DataIdT,
    TransformationData, TransformationPlan,
};
use crate::{
    feature::{ResourceOp, Transformation},
    InfraIdentifier, InfraManager, SeResult, Variant,
};

use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    rc::{Rc, Weak},
};

/// A TransformationContext contains all data involved in a data flow and is used to construct
/// transformation plans for data materialization.
#[derive(Serialize, Deserialize)]
pub struct TransformationContext {
    next_data_id: DataIdT,
    data_vec: Vec<Rc<dyn TransformationData>>,
    #[serde(skip_serializing, skip_deserializing)]
    transformation: Option<Transformation>,
    sink_infras: HashMap<DataIdT, InfraIdentifier>,
}

// non-pub struct makes it difficult to be used in pub trait TransformationData. I will make it pub for now until I find any better solution.
/// Used by the tranformation operations on TransformationData to create and register new data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DataTransformationContext {
    pub id: DataIdT,
    pub(crate) parent_data_ids: Vec<DataIdT>,
    pub(crate) args: CommonTransformationArgs,
    #[serde(skip_serializing, skip_deserializing)]
    pub transformation_context: Weak<RefCell<TransformationContext>>,
}

impl DataTransformationContext {
    pub fn get_infra_id(&self) -> Option<InfraIdentifier> {
        self.args.infra_id().cloned()
    }

    pub(super) fn get_transformation_args(&self) -> &CommonTransformationArgs {
        &self.args
    }

    pub(super) fn register_data(&self, data: &Rc<impl TransformationData + 'static>) {
        self.transformation_context
            .upgrade()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .add_data(data);
    }

    pub(super) fn new_data_context(
        &self,
        common_args: Option<CommonTransformationArgs>,
    ) -> DataTransformationContext {
        DataTransformationContext {
            id: self
                .transformation_context
                .upgrade()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .new_data_id(),
            parent_data_ids: vec![self.id],
            args: common_args.unwrap_or(CommonTransformationArgs::new(self.get_infra_id())),
            transformation_context: self.transformation_context.clone(),
        }
    }

    pub(super) fn get_transformation_id(&self) -> String {
        self.transformation_context
            .upgrade()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .get_transformation()
            .resource_id()
    }

    pub(super) fn export_resource(
        &self,
        data_id: DataIdT,
        resource_id: String,
        sink_infra_id: &InfraIdentifier,
    ) {
        self.transformation_context
            .upgrade()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .set_sink_infra(data_id, sink_infra_id)
            .get_transformation()
            .export_resources
            .push((data_id, resource_id))
    }
}

impl std::fmt::Debug for TransformationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, data) in self.data_vec.iter().enumerate() {
            f.write_fmt(format_args!("data {index}, {}", data.typetag_name()))?;
        }
        f.write_fmt(format_args!("num data {}", self.data_vec.len()))
    }
}

impl TransformationContext {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            next_data_id: 0,
            data_vec: Vec::new(),
            transformation: None,
            sink_infras: HashMap::new(),
        }))
    }

    pub(super) fn new_data_id(&mut self) -> DataIdT {
        self.next_data_id += 1;
        self.next_data_id - 1
    }

    pub(super) fn add_data(&mut self, data: &Rc<impl TransformationData + 'static>) {
        self.data_vec.push(data.clone());
    }

    fn set_sink_infra(&mut self, data_id: DataIdT, infra_id: &InfraIdentifier) -> &mut Self {
        self.sink_infras.insert(data_id, infra_id.clone());
        self
    }

    pub(super) fn get_transformation(&mut self) -> &mut Transformation {
        // create anonymous transformation
        self.transformation.get_or_insert(Transformation::default())
    }

    pub fn build_transformation(
        &mut self,
        name: Option<String>,
        variant: Variant,
    ) -> Result<Option<&Transformation>, Box<dyn Error>> {
        let body = serde_json::to_string(&self)?;
        if let Some(transformation) = &mut self.transformation {
            if let Some(name_str) = name {
                transformation.name = name_str;
            }
            transformation.body = body;
            transformation.variant = variant;
        };
        Ok(self.transformation.as_ref())
    }

    pub(crate) fn get_materialization_plan(&self, data_ids: Vec<DataIdT>) -> TransformationPlan {
        TransformationPlan::construct(data_ids, &self.data_vec, &self.sink_infras)
    }

    pub(super) fn set_and_validate_infras(&mut self, infra_manager: &InfraManager) -> SeResult<()> {
        /* TODO(tatiana): traverse according to topological order from source to sink to infer the
        suitable infra according to parent sink/execution infra, infra manager or config? */
        // set sink infra for each transformation data if unspecified according to its execution infra
        for data in &self.data_vec {
            self.sink_infras
                .entry(data.get_data_id())
                .or_insert_with(|| {
                    data.get_context().get_infra_id().expect(
                        "Now assuming the execution infra of all transformation data are specified",
                    )
                });
        }
        for data in &self.data_vec {
            set_validate_infras(
                infra_manager,
                &self.data_vec,
                data.get_data_id(),
                &self.sink_infras,
            )?;
        }
        Ok(())
    }
}
