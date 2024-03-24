mod functions;
mod resources;
use resources::*;

mod transformation;
pub(crate) use transformation::*;

use super::FeatureStore;
use crate::{
    feature::ResourceId, feature::ResourceOp, Entity, Field, Graph, GraphDataset, InfraIdentifier,
    TableFeatureView, TopologyFeatureView, Variant,
};
use futures::future::join_all;
use pyo3::{exceptions::PyValueError, prelude::*};
use tokio::runtime::Runtime;

#[pyclass(unsendable, module = "ofnil")]
pub(crate) struct ClientInner {
    pub(crate) rt: Runtime,
    pub(crate) fs: FeatureStore,
}

#[pymethods]
impl ClientInner {
    #[new]
    fn pydefault(ofnil_home: Option<&str>) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        Ok(Self {
            fs: rt.block_on(async {
                FeatureStore::init(ofnil_home)
                    .await
                    .map_err(|err| PyValueError::new_err(err.to_string()))
            })?,
            rt,
        })
    }

    pub fn register_graph(
        self_: PyRef<Self>,
        graph_name: String,
        entities: Vec<Entity>,
        fields: Vec<Field>,
        infra: Option<InfraIdentifier>,
        variant: Option<String>,
    ) -> PyResult<Graph> {
        let graph = Graph::new(
            &graph_name,
            match variant {
                Some(str) => Variant::user_defined(str),
                None => Variant::default(),
            },
            entities.iter().collect(),
            infra,
        );
        let fields = fields
            .into_iter()
            .map(|f| Field {
                sink_infra_id: graph.sink_infra_id.clone(),
                ..f
            })
            .collect();
        self_
            .rt
            .block_on(self_.register_graph_inner(graph, entities, fields))
            .map_err(|e| PyValueError::new_err(format!("Error when register_graph(): {e}")))
    }

    pub fn get_graph_dataset(self_: PyRef<Self>, graph_id: ResourceId) -> PyResult<PyObject> {
        self_.rt.block_on(async {
            match self_.fs.registry.get::<GraphDataset>(&graph_id).await {
                Ok(graph) => {
                    let mut topos = Vec::new();
                    let mut tables = Vec::new();
                    for view in graph.table_feature_views {
                        tables.push(
                            self_
                                .create_feature_view_info(view)
                                .await?
                                .into_py(self_.py()),
                        );
                    }
                    for view in graph.topology_feature_views {
                        topos.push(self_.create_topo_view_info(view).await?.into_py(self_.py()));
                    }
                    Ok((topos, tables).into_py(self_.py()))
                }
                Err(e) => Err(PyValueError::new_err(format!(
                    "Cannot get graph dataset {graph_id}. {e}"
                ))),
            }
        })
    }

    pub fn get_feature_view(self_: PyRef<Self>, view_id: ResourceId) -> PyResult<PyObject> {
        self_.rt.block_on(async {
            match self_.fs.registry.get_table_feature_view(&view_id).await {
                Ok(view) => Ok(self_
                    .create_feature_view_info(view)
                    .await?
                    .into_py(self_.py())),
                Err(e) => Err(PyValueError::new_err(format!(
                    "Cannot get table feature view {view_id}. {e}"
                ))),
            } // get view
        })
    }

    pub fn get_topology_view(self_: PyRef<Self>, view_id: ResourceId) -> PyResult<PyObject> {
        self_.rt.block_on(async {
            match self_.fs.registry.get_topology_feature_view(&view_id).await {
                Ok(topo_view) => Ok(self_
                    .create_topo_view_info(topo_view)
                    .await?
                    .into_py(self_.py())),
                Err(e) => Err(PyValueError::new_err(format!(
                    "Cannot get topology feature view {view_id}. {e}"
                ))),
            }
        })
    }
}

impl ClientInner {
    async fn register_graph_inner(
        &self,
        graph: Graph,
        entities: Vec<Entity>,
        fields: Vec<Field>,
    ) -> Result<Graph, Box<dyn std::error::Error>> {
        let refs = entities.iter().collect();
        self.fs.registry.register_resources(&refs).await?;
        let refs = fields.iter().collect();
        self.fs.registry.register_resources(&refs).await?;
        self.fs.registry.register_resource(&graph).await?;
        Ok(graph)
    }

    async fn create_topo_view_info(
        &self,
        topo_view: TopologyFeatureView,
    ) -> PyResult<TopologyFeatureViewInfo> {
        // get topology definitions
        let topologies: Vec<_> = join_all(
            topo_view
                .topology_ids
                .iter()
                .map(|id| self.fs.registry.get_topology(id)),
        )
        .await;
        // get src, dst, and edge entities
        let mut topo_tuples = Vec::new();
        for topo_res in topologies {
            match topo_res {
                Ok(topo) => {
                    let entities = self
                        .fs
                        .registry
                        .get_entities(vec![
                            topo.src_node_entity_id.as_ref().unwrap(),
                            topo.dst_node_entity_id.as_ref().unwrap(),
                            topo.edge_entity_id.as_ref().unwrap(),
                        ])
                        .await
                        .unwrap();
                    topo_tuples.push((
                        entities[0].to_owned(),
                        entities[1].to_owned(),
                        entities[2].to_owned(),
                        topo,
                    ));
                }
                Err(e) => {
                    return Err(PyValueError::new_err(format!(
                        "Cannot get Topology in {}. {}",
                        topo_view.resource_id(),
                        e
                    )));
                }
            }
        }
        Ok(TopologyFeatureViewInfo::new(
            &self.fs.infra_manager,
            topo_tuples,
            topo_view,
        ))
    }

    async fn create_feature_view_info(
        &self,
        view: TableFeatureView,
    ) -> PyResult<TableFeatureViewInfo> {
        let fields = join_all(
            view.field_ids
                .iter()
                .map(|field_id| self.fs.registry.get_field(field_id)),
        )
        .await
        .into_iter()
        .map(|res| res.unwrap())
        .collect();
        match self.fs.registry.get_entity(&view.entity_id).await {
            Ok(entity) => Ok(TableFeatureViewInfo::new(
                &self.fs.infra_manager,
                view,
                entity,
                fields,
            )),
            Err(e) => Err(PyValueError::new_err(format!(
                "Cannot get entity {} in table feature view {}. {}",
                view.entity_id,
                view.resource_id(),
                e
            ))),
        } // get entity
    }
}

#[pymodule]
fn ofnil(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<ClientInner>()?;
    module.add_class::<TableFeatureViewInfo>()?;
    module.add_class::<TopologyFeatureViewInfo>()?;
    module.add_class::<transformation::PyPipelineContext>()?;
    module.add_class::<transformation::PyGraphFrame>()?;
    module.add_class::<transformation::PyDataFrame>()?;
    module.add_function(wrap_pyfunction!(functions::neighbor_sample, module)?)?;
    crate::feature::init_module(module)?;
    crate::serving::init_module(module)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entity, fields, FeatureValueType, Topology, TopologyType};

    async fn build_test_context(
        fs: &FeatureStore,
    ) -> Result<(Vec<Entity>, Vec<Vec<Field>>, Vec<Topology>), Box<dyn std::error::Error>> {
        // vertices
        let product = entity!("test_product", Variant::Default(), "Product", "asin");
        let reviewer = entity!(
            "test_reviewer",
            Variant::Default(),
            "Reviewer",
            "reviewerID"
        );

        // edges
        let rates = entity!(
            "test_rates",
            Variant::Default(),
            "rates",
            &reviewer,
            &product
        );

        let same_rates = entity!(
            "test_same_rates",
            Variant::Default(),
            "sameRates",
            &reviewer,
            &reviewer
        );

        fs.registry
            .register_resources(&vec![&product, &reviewer, &rates, &same_rates])
            .await
            .expect("register entities");

        // fields
        // TODO(tatiana): rename the macro to fields!
        let mut product_fields = fields!(
            vec![
                ("asin", FeatureValueType::String),
                ("price", FeatureValueType::Float),
            ],
            &product,
            Variant::Default(),
            Some(InfraIdentifier::Redis("redis".to_owned())),
        );
        product_fields.iter_mut().for_each(|field| {
            field.sink_infra_id = Some(InfraIdentifier::Redis("redis".to_owned()));
        });
        let mut reviewer_fields = fields!(
            vec![("reviewerID", FeatureValueType::String)],
            &reviewer,
            Variant::Default(),
            Some(InfraIdentifier::Redis("redis".to_owned())),
        );
        reviewer_fields.iter_mut().for_each(|field| {
            field.sink_infra_id = Some(InfraIdentifier::Redis("redis".to_owned()));
        });

        fs.registry
            .register_resources(&product_fields.iter().chain(&reviewer_fields).collect())
            .await
            .expect("register fields");

        let usu = Topology {
            name: "usu".to_owned(),
            transformation_id: None,
            topology_type: Some(TopologyType::AdjacencyMatrix),
            sink_infra_id: Some(crate::InfraIdentifier::Neo4j("neo4j_1".to_owned())),
            edge_entity_id: Some(same_rates.resource_id()),
            src_node_entity_id: Some(reviewer.resource_id()),
            dst_node_entity_id: Some(reviewer.resource_id()),
            ..Default::default()
        };
        let u2i = Topology {
            name: "utoi".to_owned(),
            transformation_id: None,
            topology_type: Some(TopologyType::AdjacencyMatrix),
            sink_infra_id: Some(crate::InfraIdentifier::Neo4j("neo4j_1".to_owned())),
            edge_entity_id: Some(rates.resource_id()),
            src_node_entity_id: Some(reviewer.resource_id()),
            dst_node_entity_id: Some(product.resource_id()),
            ..Default::default()
        };
        fs.registry
            .register_resources(&vec![&usu, &u2i])
            .await
            .expect("register topology");

        Ok((
            vec![product, reviewer, rates, same_rates],
            vec![product_fields, reviewer_fields],
            vec![usu, u2i],
        ))
    }

    #[tokio::test]
    async fn test_py_topo_feature_view() -> Result<(), Box<dyn std::error::Error>> {
        use std::path::PathBuf;
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("examples/quickstart/");
        std::env::set_var("OFNIL_HOME", dir.to_str().unwrap());
        let fs = FeatureStore::init(None)
            .await
            .expect("Build connection with feature store");
        let (_, _, topos) = build_test_context(&fs)
            .await
            .expect("Register all resources");

        let view = TopologyFeatureView::default("test_py_topo_view", &topos);
        fs.registry.register_resource(&view).await?;
        // pytest
        let test_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/python/tests/infra/graph/test_neo4j_neighbor_sampler.py"
        );
        let o = std::process::Command::new("pytest")
            .arg(test_path)
            .arg("-k")
            .arg("test_neo4j_pyg_sampler_hetero")
            .output()
            .unwrap();
        println!("stdout: {}", String::from_utf8_lossy(&o.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success());
        Ok(())
    }

    #[tokio::test]
    async fn test_py_table_feature_view() -> Result<(), Box<dyn std::error::Error>> {
        use std::path::PathBuf;
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("examples/quickstart/");
        std::env::set_var("OFNIL_HOME", dir.to_str().unwrap());
        let fs = FeatureStore::init(None)
            .await
            .expect("Build connection with feature store");
        let (entities, fields, topos) = build_test_context(&fs)
            .await
            .expect("Register all resources");

        let view = TopologyFeatureView::default("test_py_topo_view", &topos);
        fs.registry.register_resource(&view).await?;
        let view = TableFeatureView::default(
            "test_py_table_view",
            entities[0].resource_id(),
            &vec![fields[0][1].clone()],
        );
        fs.registry.register_resource(&view).await?;
        // pytest
        let test_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/python/tests/torch/test_neighbor_sampled_dataloader.py"
        );
        // PyG
        let o = std::process::Command::new("pytest")
            .arg(test_path)
            .arg("--capture=no")
            .arg("-k")
            .arg("test_neo4j_pyg_sampler_hetero")
            .output()
            .unwrap();
        println!("stdout: {}", String::from_utf8_lossy(&o.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success());
        // DGL
        let o = std::process::Command::new("pytest")
            .arg(test_path)
            .arg("--capture=no")
            .arg("-k")
            .arg("test_neo4j_dgl_sampler_hetero")
            .output()
            .unwrap();
        println!("stdout: {}", String::from_utf8_lossy(&o.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&o.stderr));
        assert!(o.status.success());
        Ok(())
    }
}
