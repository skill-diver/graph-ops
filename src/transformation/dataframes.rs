use super::{built_in_fns::expression::Expression, TransformationData};
use crate::{
    feature::{ResourceId, ResourceOp},
    Field, InfraIdentifier,
};

use std::collections::HashMap;
use std::rc::Rc;

mod column;
pub use column::Column;

mod dataframe;
pub use dataframe::DataFrame;

mod vertex_feature_dataframe;
pub use vertex_feature_dataframe::VertexFeatureDataFrame;

mod aggregate_dataframe;
pub use aggregate_dataframe::{AggregateDataFrame, AggregateError, AggregateFunc};

mod dataframe_inner;
pub(crate) use dataframe_inner::DataFrameInner;

pub trait DataFrameBase: TransformationData {
    fn get_inner(&self) -> &DataFrameInner;
    fn entity_id(&self) -> Option<ResourceId>;

    // default implementation
    fn select(&self, expr: Vec<String>) -> Rc<DataFrame> {
        let schema = expr
            .iter()
            .map(|e| {
                let encoder = Expression::new(e); // TODO(tatiana): need to parse the expression
                let mut col = Column {
                    origin: self.get_context().id,
                    expr: Some(e.to_string()),
                    encoder: None,
                    value_type: encoder.get_type(),
                };
                col.encoder = Some(encoder);
                Rc::new(col)
            })
            .collect::<Vec<_>>();
        let col_names = schema
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                col.encoder
                    .as_ref()
                    .unwrap()
                    .get_col_name()
                    .unwrap_or(format!("col{idx}"))
            })
            .collect();
        let new_data_context = self.get_context().new_data_context(None);
        let res = Rc::new(DataFrame {
            inner: DataFrameInner::new(self.name(), new_data_context, schema, col_names),
            entity_id: None,
        });
        self.get_context().register_data(&res);
        res
    }

    fn with_column(&self, colname: &str, col: Rc<Column>) -> Rc<DataFrame> {
        // TODO(tatiana): need to check the origin of the column must be compatible with this
        // Dataframe (i.e. same size, same indexing)
        let mut new_schema = self.schema().clone();
        let mut new_col_names = self.col_names().clone();
        let new_data_context = self.get_context().new_data_context(None);
        new_schema.push(col);
        new_col_names.push(colname.to_string());
        let res = Rc::new(DataFrame {
            inner: DataFrameInner::new(self.name(), new_data_context, new_schema, new_col_names),
            entity_id: None,
        });
        self.get_context().register_data(&res);
        res
    }

    fn col(&self, colname: &str) -> Option<Rc<Column>> {
        self.col_by_names()
            .get(colname)
            .map(|entry| self.schema()[*entry].clone())
    }

    fn export(&self, sink_infra_id: &InfraIdentifier) -> Vec<Field> {
        let res: Vec<Field> = self
            .schema()
            .iter()
            .enumerate()
            .map(|(i, col)| {
                col.to_field(
                    &self.col_names()[i],
                    self.get_context().get_transformation_id(),
                    self.entity_id(),
                    Some(sink_infra_id),
                )
            })
            .collect();
        res.iter().for_each(|e| {
            self.get_context()
                .export_resource(self.get_data_id(), e.resource_id(), sink_infra_id);
        });
        res
    }

    fn name(&self) -> &str {
        &self.get_inner().name
    }
    fn schema(&self) -> &Vec<Rc<Column>> {
        &self.get_inner().schema
    }
    fn col_names(&self) -> &Vec<String> {
        &self.get_inner().col_names
    }
    fn col_by_names(&self) -> &HashMap<String, usize> {
        &self.get_inner().col_by_names
    }
}
