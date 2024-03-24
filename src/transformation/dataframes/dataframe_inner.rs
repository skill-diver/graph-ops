use super::Column;
use crate::transformation::transformation_context::DataTransformationContext;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataFrameInner {
    pub(super) context: DataTransformationContext,
    pub(super) name: String,
    pub(super) schema: Vec<Rc<Column>>, // from meta or transformation
    pub(super) col_names: Vec<String>,  // from meta or transformation
    pub(super) col_by_names: HashMap<String, usize>,
}

impl DataFrameInner {
    pub(crate) fn new(
        name: impl Into<String>,
        context: DataTransformationContext,
        schema: Vec<Rc<Column>>,
        col_names: Vec<String>,
    ) -> Self {
        let col_by_names = col_names
            .iter()
            .enumerate()
            .map(|(v, k)| (k.clone(), v))
            .collect();
        Self {
            context,
            name: name.into(),
            schema,
            col_names,
            col_by_names,
        }
    }
}
