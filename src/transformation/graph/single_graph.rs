use super::{DataFrameSet, Selector, VertexSelectGraph};
use crate::{
    feature::{ResourceId, ResourceOp},
    infra::pi::{TransformationConnector, GAF},
    FeatureRegistry, Field, Graph, SeResult, Topology, TopologyType, Variant,
};
use crate::{
    transformation::{
        transformation_plan::{TransformationIOT, TransformationOp, TransformationOutputHandler},
        CommonTransformationArgs, DataIdT, DataTransformationContext, GraphBase,
        TransformationContext, TransformationData,
    },
    Entity, GraphDataset, InfraIdentifier, InfraManager,
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

/// A single (large) graph containing one or multiple types of vertices.
#[derive(Debug, Serialize, Deserialize)]
pub struct SingleGraph {
    context: DataTransformationContext,
    vertex_fvs: HashMap<String, (String, Vec<Field>)>, // tlabel, (view name, fields)
    edge_fvs: HashMap<String, (String, Vec<Field>)>,   // tlabel, (view name, fields)
    vertex_entities: HashMap<String, ResourceId>,      // tlabel, entity id
    edge_entities: HashMap<String, ResourceId>,        // tlabel, entity id
    topology_type: Option<TopologyType>,
}

// TODO(tatiana): refactor constructors
impl SingleGraph {
    pub fn new(
        context: &Rc<RefCell<TransformationContext>>,
        vertex_entity_fields: Vec<(Entity, Vec<Field>)>,
        edge_entity_fields: Vec<(Entity, Vec<Field>)>,
        infra_id: InfraIdentifier,
    ) -> Rc<Self> {
        let vertex_entities = vertex_entity_fields
            .iter()
            .map(|(entity, _)| (entity.tlabel().to_string(), entity.resource_id()))
            .collect();
        let edge_entities = edge_entity_fields
            .iter()
            .map(|(entity, _)| (entity.tlabel().to_string(), entity.resource_id()))
            .collect();
        let res = Rc::new(Self {
            context: DataTransformationContext {
                id: context.as_ref().borrow_mut().new_data_id(),
                args: CommonTransformationArgs::new(Some(infra_id)),
                transformation_context: Rc::downgrade(context),
                parent_data_ids: Vec::new(),
            },
            vertex_fvs: vertex_entity_fields
                .into_iter()
                .map(|(entity, fields)| {
                    let view_name = format!("{}_ALL_FIELDS", entity.name());
                    (entity.tlabel().to_string(), (view_name, fields))
                })
                .collect(),
            edge_fvs: edge_entity_fields
                .into_iter()
                .map(|(entity, fields)| {
                    let view_name = format!("{}_ALL_FIELDS", entity.name());
                    (entity.tlabel().to_string(), (view_name, fields))
                })
                .collect(),
            vertex_entities,
            edge_entities,
            topology_type: None,
        });
        context.as_ref().borrow_mut().add_data(&res);
        res
    }

    pub async fn from(
        context: &Rc<RefCell<TransformationContext>>,
        meta: &GraphDataset,
        registry: &FeatureRegistry,
    ) -> Result<Rc<SingleGraph>, Box<dyn Error>> {
        let id = context.as_ref().borrow_mut().new_data_id();
        let mut res = SingleGraph {
            context: DataTransformationContext {
                id,
                args: CommonTransformationArgs::default(),
                transformation_context: Rc::downgrade(context),
                parent_data_ids: Vec::new(),
            },
            vertex_fvs: HashMap::new(),
            edge_fvs: HashMap::new(),
            vertex_entities: HashMap::new(),
            edge_entities: HashMap::new(),
            topology_type: None,
        };
        for view in &meta.table_feature_views {
            let entity = registry.get_entity(&view.entity_id).await?;
            let mut fields = Vec::new();
            for id in &view.field_ids {
                let field = registry.get_field(id).await?;
                fields.push(field);
            }
            match &entity {
                Entity::Vertex(entity) => {
                    // now treat entity name as vertex type
                    res.vertex_fvs
                        .insert(entity.tlabel.clone(), (view.name.clone(), fields));
                    res.vertex_entities
                        .insert(entity.tlabel.clone(), view.entity_id.clone());
                }
                Entity::Edge(entity) => {
                    // now treat entity name as edge type
                    res.edge_fvs
                        .insert(entity.tlabel.clone(), (view.name.clone(), fields));
                    res.edge_entities
                        .insert(entity.tlabel.clone(), view.entity_id.clone());
                }
            };
        }
        for view in &meta.topology_feature_views {
            res.topology_type = Some(view.topology_type.clone());
        }

        let res = Rc::new(res);
        context.as_ref().borrow_mut().add_data(&res);
        Ok(res)
    } // pub fn from
}

impl Graph {
    pub async fn transform(
        &self,
        context: &Rc<RefCell<TransformationContext>>,
        registry: &FeatureRegistry,
    ) -> Result<Rc<SingleGraph>, Box<dyn Error>> {
        let mut vertex_fvs = HashMap::new();
        let mut edge_fvs = HashMap::new();
        let mut vertex_entities = HashMap::new();
        let mut edge_entities = HashMap::new();
        for id in self.entity_ids.values() {
            let entity = registry.get_entity(id).await?;
            let entity_name = Entity::id_to_name(id);
            let view_name = format!("{entity_name}_ALL_FIELDS");
            // list all fields of each entity
            let fields = registry
                .get_entity_fields(entity_name, &Variant::default())
                .await?;
            match &entity {
                Entity::Vertex(entity) => {
                    vertex_fvs.insert(entity.tlabel.clone(), (view_name, fields));
                    vertex_entities.insert(entity.tlabel.clone(), id.clone());
                }
                Entity::Edge(entity) => {
                    edge_fvs.insert(entity.tlabel.clone(), (view_name, fields));
                    edge_entities.insert(entity.tlabel.clone(), id.clone());
                }
            }
        }

        let id = context.as_ref().borrow_mut().new_data_id();
        let res = Rc::new(SingleGraph {
            context: DataTransformationContext {
                id,
                args: CommonTransformationArgs::new(self.sink_infra_id.clone()),
                transformation_context: Rc::downgrade(context),
                parent_data_ids: Vec::new(),
            },
            vertex_fvs,
            edge_fvs,
            vertex_entities,
            edge_entities,
            topology_type: None,
        });
        context.as_ref().borrow_mut().add_data(&res);
        Ok(res)
    } // fn transform
}

#[pymethods]
impl Graph {
    #[pyo3(name = "transform")]
    fn pytransform(
        self_: PyRef<Self>,
        context: PyRef<crate::python::PyPipelineContext>,
    ) -> PyResult<crate::python::PyGraphFrame> {
        let client = context.client.borrow(self_.py());
        client
            .rt
            .block_on(self_.transform(&context.inner, &client.fs.registry))
            .map(|rc| crate::python::PyGraphFrame { inner: rc })
            .map_err(|e| PyRuntimeError::new_err(format!("Error in Graph.transform(): {e}")))
    }
}

#[derive(Debug)]
pub(super) struct GraphDatabaseSourceOp {
    pub common_args: CommonTransformationArgs,
    pub execution_infra: Option<Box<dyn TransformationConnector>>, // required for execution
}

#[async_trait::async_trait(?Send)]
impl TransformationOp for GraphDatabaseSourceOp {
    async fn execute(
        &self,
        _data_id: DataIdT,
        _input: &TransformationIOT,
    ) -> SeResult<TransformationOutputHandler> {
        // TODO(tatiana): for now, we assume only one graph instance in the graph database, so that cypher queries on this instance need no input
        // if dataframe is to be retrived from the graph, this execute function should execute a graph query to prepare the needed properties.

        Ok(TransformationOutputHandler::InfraHandler {
            infra_id: self.common_args.infra_id().cloned().unwrap(),
        })
    }

    fn get_common_args(&self) -> &CommonTransformationArgs {
        &self.common_args
    }

    fn set_execution_connector(&mut self, infra_manager: &InfraManager) {
        self.execution_infra =
            Some(infra_manager.get_graph_transformation_infra_cloned(
                self.common_args.infra_id().as_ref().unwrap(),
            ))
            .unwrap()
    }

    fn get_execution_connector(&self) -> &dyn TransformationConnector {
        self.execution_infra.as_ref().unwrap().as_ref()
    }
}

#[typetag::serde]
impl TransformationData for SingleGraph {
    fn get_context(&self) -> &DataTransformationContext {
        &self.context
    }

    fn get_producer_op(&self) -> Box<dyn TransformationOp> {
        Box::new(GraphDatabaseSourceOp {
            common_args: self.context.get_transformation_args().clone(),
            execution_infra: None,
        })
    }

    fn get_func(&self) -> GAF {
        GAF::Source
    }
}

impl GraphBase for SingleGraph {
    fn get_vertex_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        &self.vertex_fvs
    }

    fn get_edge_fvs(&self) -> &HashMap<String, (String, Vec<Field>)> {
        &self.edge_fvs
    }

    fn get_vertex_entities(&self) -> &HashMap<String, ResourceId> {
        &self.vertex_entities
    }

    fn get_edge_entities(&self) -> &HashMap<String, ResourceId> {
        &self.edge_entities
    }

    fn get_topology_type(&self) -> &Option<TopologyType> {
        &self.topology_type
    }

    fn vertices(&self) -> Rc<dyn GraphBase> {
        let res = Rc::new(VertexSelectGraph {
            context: self.get_context().new_data_context(None),
            graph: self.get_data_id(),
            selector: Selector::DirectAccess { ltype: None },
            df: if self.get_vertex_fvs().len() == 1 {
                DataFrameSet::Homo(self.get_edge_fvs().values().next().unwrap().clone())
            } else {
                DataFrameSet::Hetero(self.get_vertex_fvs().clone())
            },
        });
        self.get_context().register_data(&res);
        res
    }

    fn edges(&self) -> Rc<dyn GraphBase> {
        todo!()
    }

    fn vertices_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        if let Some(df) = self.get_vertex_fvs().get(t) {
            let res = Rc::new(VertexSelectGraph {
                context: self.get_context().new_data_context(None),
                graph: self.get_data_id(),
                selector: Selector::DirectAccess {
                    ltype: Some(t.to_string()),
                },
                df: DataFrameSet::Homo(df.clone()),
            });
            self.get_context().register_data(&res);
            Some(res)
        } else {
            None
        }
    }

    #[allow(unused)] // TODO(tatiana): implementation
    fn edges_by_type(&self, t: &str) -> Option<Rc<dyn GraphBase>> {
        todo!()
    }

    // TODO(tatiana): return a vec of Topology?
    fn export_topology(&self, name: &str, sink_infra_id: &InfraIdentifier) -> Topology {
        let res = Topology {
            name: name.to_string(),
            transformation_id: Some(self.get_context().get_transformation_id()),
            topology_type: self.get_topology_type().clone(),
            sink_infra_id: Some(sink_infra_id.clone()),
            ..Default::default()
        };
        // TODO(tatiana): fill in entities

        self.get_context()
            .export_resource(self.get_data_id(), res.resource_id(), sink_infra_id);
        res
    }
}
