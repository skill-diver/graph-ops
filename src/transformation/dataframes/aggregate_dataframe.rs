use super::{dataframe_inner::DataFrameInner, DataFrameBase};
use crate::{
    feature::{ResourceId, ResourceOp},
    infra::pi::GAF,
    transformation::{
        built_in_fns::aggregate_neighbor_args::AggregateNeighborArgs,
        transformation_plan::AggregateOp, BuiltInFnArgs, DataTransformationContext,
        GraphProjectionArgs, TransformationArgs, TransformationData, TransformationOp,
    },
    Entity,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AggregateFunc {
    Count,
    Sum,
    Mean,
    Min,
    Max,
    Std,
}

#[derive(thiserror::Error, Debug)]
pub enum AggregateError {
    #[error("Ofnil do not support {0} aggregation.")]
    AggregateFunctionError(String),
}

impl AggregateFunc {
    pub fn as_cypher_str(&self) -> &str {
        match self {
            AggregateFunc::Count => "count",
            AggregateFunc::Sum => "sum",
            AggregateFunc::Mean => "avg",
            AggregateFunc::Min => "min",
            AggregateFunc::Max => "max",
            AggregateFunc::Std => "std",
        }
    }
}

/// AggregateDataFrame is a DataFrame that contains the information of an aggregation operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateDataFrame {
    inner: DataFrameInner,
    func: AggregateFunc,
    edge_entity: Option<Entity>, // can get tlabel, src_tlabel, dst_tlabel
    target_vertex_entity: Entity,
    properties: Vec<String>,
}

impl AggregateDataFrame {
    pub fn new(
        dataframe_inner: DataFrameInner,
        aggregator: AggregateFunc,
        edge_entity: Option<Entity>,
        target_vertex_entity: Entity,
        properties: Vec<String>,
    ) -> Self {
        Self {
            inner: dataframe_inner,
            func: aggregator,
            edge_entity,
            target_vertex_entity,
            properties,
        }
    }
}

impl DataFrameBase for AggregateDataFrame {
    fn get_inner(&self) -> &DataFrameInner {
        &self.inner
    }
    fn entity_id(&self) -> Option<ResourceId> {
        Some(self.target_vertex_entity.resource_id())
    }
}

#[typetag::serde]
impl TransformationData for AggregateDataFrame {
    fn get_context(&self) -> &DataTransformationContext {
        &self.inner.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        let (edge_entity, src_tlabel, dst_tlabel) = match &self.edge_entity {
            Some(e) => match e {
                Entity::Edge(entity) => (
                    entity.clone(),
                    entity.src_tlabel.to_owned(),
                    entity.dst_tlabel.to_owned(),
                ),
                _ => panic!("AggregateDataFrame edge_entity is not an edge entity"),
            },
            _ => panic!("AggregateDataFrame edge_entity is not set"),
        };

        let target_node_tlabel = match &self.target_vertex_entity {
            Entity::Vertex(entity) => entity.tlabel.to_owned(),
            _ => panic!("AggregateDataFrame dst_node_entity is not a node entity"),
        };

        let target_node_primary_key = self.target_vertex_entity.primary_key().unwrap().to_owned();

        Box::new(AggregateOp::new(
            TransformationArgs::new_vertex_feature_args(
                BuiltInFnArgs::AggregateNeighbor(AggregateNeighborArgs {
                    func: self.func,
                    properties: self.properties.clone(),
                }),
                GraphProjectionArgs {
                    vertices: vec![(src_tlabel, None), (dst_tlabel, None)],
                    edges: vec![edge_entity],
                    make_edges_undirected: false,
                },
                target_node_tlabel,
                target_node_primary_key,
                self.inner.col_names.clone(),
            ),
            self.inner.context.get_transformation_args().clone(),
        ))
    }

    fn get_func(&self) -> GAF {
        GAF::OneOf(vec![GAF::AggregateNeighbors, GAF::Cypher])
    }
}
