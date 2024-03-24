use crate::{
    feature::ResourceId,
    infra::pi::GAF,
    transformation::{
        built_in_fns::{expression::Expression, sampling::SamplingSpec},
        transformation_plan::TransformationOp,
        DataIdT, DataTransformationContext, GraphBase, TransformationData,
    },
    Field, InfraIdentifier, Topology, TopologyType,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

/// The representation of vertex/edge selection, recursively defined as a union of direct access, filtering, and sampling results
#[derive(Serialize, Deserialize)]
pub enum Selector {
    /// Direct access to the set of vertices/edges in a graph
    DirectAccess {
        ltype: Option<String>,
    },
    /// A set of vertices/edges in a graph whose expression is evaluated to true
    Expression(Expression),
    /// A set of sampled vertices/edges in a graph
    Sampling(SamplingSpec),
    Union(Box<Selector>, Box<Selector>),
}

#[derive(Serialize, Deserialize)]
pub enum DataFrameSet {
    Homo((String, Vec<Field>)),                    // name, fields
    Hetero(HashMap<String, (String, Vec<Field>)>), // {type, {name, fields}}
}

/// Represents a filter-projection vertex relation from the original graph, such as
/// select vertices {feat1, feat3, feat2 / feat2.avg()} from graph where vertices.type = "Person"
#[derive(Serialize, Deserialize)]
pub struct VertexSelectGraph {
    pub(super) context: DataTransformationContext,
    pub(super) graph: DataIdT,     // graph data id
    pub(super) selector: Selector, // vertex selection
    pub(super) df: DataFrameSet,
}

impl VertexSelectGraph {
    #[allow(dead_code)] // TODO(tatiana): to be used elsewhere
    pub fn new(
        context: DataTransformationContext,
        graph: DataIdT,
        selector: Selector,
        df: DataFrameSet,
    ) -> Self {
        VertexSelectGraph {
            context,
            graph,
            selector,
            df,
        }
    }
}

/// Represents a filter-projection edge relation from the original graph
#[derive(Serialize, Deserialize)]
pub struct EdgeSelectGraph {
    pub(super) context: DataTransformationContext,
    pub(super) graph: DataIdT,     // graph data id
    pub(super) selector: Selector, // edge selection
    pub(super) df: DataFrameSet,
}

impl EdgeSelectGraph {
    #[allow(dead_code)] // TODO(tatiana): to be finished
    pub fn new(
        context: DataTransformationContext,
        graph: DataIdT,
        selector: Selector,
        df: DataFrameSet,
    ) -> Self {
        EdgeSelectGraph {
            context,
            graph,
            selector,
            df,
        }
    }
}

#[typetag::serde]
impl TransformationData for VertexSelectGraph {
    fn get_context(&self) -> &DataTransformationContext {
        &self.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        todo!()
    }

    fn get_func(&self) -> GAF {
        todo!()
    }
}

impl GraphBase for VertexSelectGraph {
    fn get_vertex_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        todo!()
    }

    fn get_edge_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        todo!()
    }

    fn get_vertex_entities(&self) -> &HashMap<String, ResourceId> {
        todo!()
    }

    fn get_edge_entities(&self) -> &HashMap<String, ResourceId> {
        todo!()
    }

    fn get_topology_type(&self) -> &Option<TopologyType> {
        todo!()
    }

    fn vertices(&self) -> Rc<dyn GraphBase> {
        // let res = Rc::new(VertexSelectGraph {
        //     context: self.get_context().new_data_context(),
        //     graph: self.get_data_id(),
        //     selector: Selector::DirectAccess { ltype: None },
        //     df: if self.get_vertex_fvs().len() == 1 {
        //         DataFrameSet::Homo(self.get_edge_fvs().values().next().unwrap().clone())
        //     } else {
        //         DataFrameSet::Hetero(self.get_vertex_fvs().clone())
        //     },
        // });
        // self.get_context().register_data(&res);
        // res
        todo!()
    }

    fn edges(&self) -> Rc<dyn GraphBase> {
        todo!()
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn vertices_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        todo!()
        // if let Some(df) = self.get_vertex_fvs().get(t) {
        //     let res = Rc::new(VertexSelectGraph {
        //         context: self.get_context().new_data_context(),
        //         graph: self.get_data_id(),
        //         selector: Selector::DirectAccess {
        //             ltype: Some(t.to_string()),
        //         },
        //         df: DataFrameSet::Homo(df.clone()),
        //     });
        //     self.get_context().register_data(&res);
        //     Some(res)
        // } else {
        //     None
        // }
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn edges_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        todo!()
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn export_topology(&self, name: &str, sink_infra_id: &InfraIdentifier) -> Topology {
        todo!()
        // let res = Topology {
        //     name: name.to_string(),
        //     transformation_id: Some(self.get_context().get_transformation_id()),
        //     topology_type: self.get_topology_type().clone(),
        //     sink_infra_id: Some(sink_infra_id.clone()),
        //     ..Default::default()
        // };

        // self.get_context()
        //     .export_resource(self.get_data_id(), res.resource_id());
        // res
    }
}
