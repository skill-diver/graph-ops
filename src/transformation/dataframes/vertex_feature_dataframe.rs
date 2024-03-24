use super::{dataframe_inner::DataFrameInner, Column, DataFrameBase};
use crate::{
    feature::{ResourceId, ResourceOp},
    infra::pi::GAF,
    transformation::{
        built_in_fns::BuiltInFnArgs,
        transformation_context::DataTransformationContext,
        transformation_plan::{BuiltInOp, TransformationOp},
        GraphProjectionArgs, TransformationArgs, TransformationData,
    },
    Entity,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub struct VertexFeatureDataFrame {
    inner: DataFrameInner,
    /// the graph transformation function used to compute the feature
    func: GAF,
    /// the algorithmic args
    fn_args: BuiltInFnArgs,
    /// the target vertex entity for which the feature is computed
    target_vertex_entity: Entity,
    /// the entities needed in graph projection
    projection_entities: Vec<Entity>,
    /// whether to make edges undirected in the projected graph
    make_edges_undirected: bool,
}

impl VertexFeatureDataFrame {
    pub fn new(
        name: impl Into<String>,
        context: DataTransformationContext,
        schema: Vec<Rc<Column>>,
        col_names: Vec<String>,
        target_vertex_entity: Entity,
        fn_args: BuiltInFnArgs,
        projection: (Vec<Entity>, bool),
    ) -> Self {
        Self {
            inner: DataFrameInner::new(name, context, schema, col_names),
            func: fn_args.get_func(),
            fn_args,
            target_vertex_entity,
            projection_entities: projection.0,
            make_edges_undirected: projection.1,
        }
    }
}

impl DataFrameBase for VertexFeatureDataFrame {
    fn get_inner(&self) -> &DataFrameInner {
        &self.inner
    }
    fn entity_id(&self) -> Option<ResourceId> {
        Some(self.target_vertex_entity.resource_id())
    }
}

#[typetag::serde]
impl TransformationData for VertexFeatureDataFrame {
    fn get_context(&self) -> &DataTransformationContext {
        &self.inner.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        let (target_vertex_tlabel, target_vertex_primary_key) = match &self.target_vertex_entity {
            Entity::Vertex(entity) => (entity.tlabel.to_owned(), entity.primary_key.to_owned()),
            _ => panic!("VertexFeatureDataFrame: target_vertex_entity is not a vertex entity"),
        };

        Box::new(BuiltInOp::new(
            self.func.clone(),
            TransformationArgs::new_vertex_feature_args(
                self.fn_args.clone(),
                GraphProjectionArgs::new(&self.projection_entities, self.make_edges_undirected),
                target_vertex_tlabel,
                target_vertex_primary_key,
                self.inner.col_names.clone(),
            ),
            self.inner.context.get_transformation_args().clone(),
        ))
    }

    fn get_func(&self) -> GAF {
        self.func.clone()
    }
}
