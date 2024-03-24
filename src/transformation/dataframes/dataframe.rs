use super::{Column, DataFrameBase, DataFrameInner};
use crate::{
    feature::ResourceId,
    infra::pi::GAF,
    transformation::{
        transformation_plan::TransformationOp, CommonTransformationArgs, DataTransformationContext,
        TransformationContext, TransformationData,
    },
    Field,
};

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// Similar to pandas DataFrame or Spark DataFrame. A DataFrame instance can be initialized from a TableFeatureView or a Field.
#[derive(Debug, Serialize, Deserialize)]
pub struct DataFrame {
    pub(super) inner: DataFrameInner,
    pub(super) entity_id: Option<ResourceId>,
}

impl DataFrame {
    #[allow(unused)]
    fn from_field(context: &Rc<RefCell<TransformationContext>>, field: &Field) -> Rc<Self> {
        DataFrame::new(context, &field.name, vec![field])
    }

    fn new(
        context: &Rc<RefCell<TransformationContext>>,
        name: &str,
        schema: Vec<&Field>,
    ) -> Rc<Self> {
        let mut tc = context.as_ref().borrow_mut();
        let id = tc.new_data_id();
        let res = Rc::new(Self {
            inner: DataFrameInner::new(
                name,
                DataTransformationContext {
                    id,
                    transformation_context: Rc::downgrade(context),
                    args: CommonTransformationArgs::default(),
                    parent_data_ids: Vec::new(),
                },
                schema
                    .iter()
                    .map(|f| {
                        Rc::new(Column {
                            origin: id,
                            expr: Some(f.name.clone()),
                            encoder: None,
                            value_type: f.value_type.clone(),
                        })
                    })
                    .collect(),
                schema.iter().map(|f| f.name.clone()).collect(),
            ),
            entity_id: None,
        });
        tc.add_data(&res);
        res
    }
}

impl DataFrameBase for DataFrame {
    fn get_inner(&self) -> &DataFrameInner {
        &self.inner
    }
    fn entity_id(&self) -> Option<ResourceId> {
        self.entity_id.clone()
    }
}

#[typetag::serde]
impl TransformationData for DataFrame {
    fn get_context(&self) -> &DataTransformationContext {
        &self.inner.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        todo!()
    }

    fn get_func(&self) -> GAF {
        GAF::Source
    }
}

#[test]
fn demo_use_dataframe() {
    use crate::{entity, fields, FeatureValueType, Variant};

    let entity = entity!("node_1", Variant::Default(), "Product", "id");

    let cols = fields!(
        vec![
            ("feature_1", FeatureValueType::Int),
            ("feature_2", FeatureValueType::Int),
            ("feature_3", FeatureValueType::Int)
        ],
        &entity,
        Variant::Default(),
        None,
    );

    let context = TransformationContext::new();
    let df = DataFrame::new(&context, "test", cols.iter().collect());
    let df2 = df.select(vec![
        "col1 + col2 * avg(col3)".to_string(),
        "col1".to_string(),
    ]);
    let _df3 = df.with_column("computed_col", df2.col("col0").unwrap());

    // println!("{:#?}", df3);
    println!("{context:#?}");

    // add test for serialization and deserialization.
    let serialized = serde_json::to_string(&*context).unwrap();
    println!("serialized = {serialized}");
    let events: Rc<RefCell<TransformationContext>> = serde_json::from_str(&serialized).unwrap();
    println!("Deserialized TransformationContext: {events:?}");
}
