use crate::{
    config::InfraConfig, serving::FeatureRenderingOptions, serving::TopologyRenderingOptions,
    Entity, Field, InfraIdentifier, InfraManager, TableFeatureView, Topology, TopologyFeatureView,
};
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

#[pyclass(name = "TableFeatureViewInfo", module = "ofnil")]
pub struct TableFeatureViewInfo {
    #[pyo3(get)]
    entity_label: String,
    #[pyo3(get)]
    primary_key: String,
    #[pyo3(get)]
    field_names: Vec<String>,
    #[pyo3(get)]
    entity_type: String, // vertex, edge, global
    #[pyo3(get, set)]
    rendering_opt: FeatureRenderingOptions,
    #[pyo3(get)]
    infra_info: HashMap<String, String>,
}

#[pyclass(name = "TopologyFeatureViewInfo", module = "ofnil")]
pub struct TopologyFeatureViewInfo {
    // [src label, dst label, edge label, Topology]
    topologies: Vec<(Entity, Entity, Entity, Topology)>,
    #[pyo3(get, set)]
    rendering_opt: TopologyRenderingOptions,
    #[pyo3(get)]
    infra_info: HashMap<String, String>,
}

impl TableFeatureViewInfo {
    pub(super) fn new(
        infra_manager: &InfraManager,
        view: TableFeatureView,
        entity: Entity,
        fields: Vec<Field>,
    ) -> Self {
        // check infra
        // now assume all fields in the same feature view are in the same storage identified by InfraIdentifier
        let mut sink_infra_id = None;
        fields.iter().for_each(|field| {
            if sink_infra_id.is_some() {
                assert!(
                    sink_infra_id == field.sink_infra_id,
                    "field {field:?} sink_infra_id is not equal to {sink_infra_id:?}"
                );
            } else {
                sink_infra_id = field.sink_infra_id.to_owned();
            }
        });
        let infra_config = if let Some(InfraIdentifier::Redis(id)) = sink_infra_id {
            infra_manager
                .get_infra_config(&InfraIdentifier::Redis(id))
                .unwrap()
        } else {
            panic!("{sink_infra_id:?} infra does not support feature view serving")
        };
        let infra_info = match infra_config {
            InfraConfig::RedisClientConfig { uri } => HashMap::from([
                ("infra_type".to_owned(), "redis".to_owned()),
                ("uri".to_owned(), uri.to_owned()),
            ]),
            _ => panic!("Expected RedisClientConfig"),
        };
        match entity {
            Entity::Vertex(entity) => Self {
                primary_key: entity.primary_key,
                field_names: fields.into_iter().map(|field| field.name).collect(),
                rendering_opt: view.rendering_opt,
                entity_label: entity.tlabel,
                entity_type: "vertex".to_owned(),
                infra_info,
            },
            Entity::Edge(entity) => Self {
                primary_key: entity
                    .primary_key
                    .expect("Edge primary key must be specified"),
                field_names: fields.into_iter().map(|field| field.name).collect(),
                rendering_opt: view.rendering_opt,
                entity_label: entity.tlabel,
                entity_type: "edge".to_owned(),
                infra_info,
            },
        }
    }
}

#[pymethods]
impl TopologyFeatureViewInfo {
    fn get_all_edge_types(&self) -> Vec<String> {
        self.topologies
            .iter()
            .map(|(_, _, edge_entity, _)| edge_entity.tlabel().to_string())
            .collect()
    }

    fn get_all_edge_types_triplet(&self) -> Vec<(String, String, String)> {
        self.topologies
            .iter()
            .map(|(src_entity, dst_entity, edge_entity, _)| {
                (
                    src_entity.tlabel().to_string(),
                    dst_entity.tlabel().to_string(),
                    edge_entity.tlabel().to_string(),
                )
            })
            .collect()
    }

    fn get_all_node_types(&self) -> Vec<String> {
        self.topologies
            .iter()
            .flat_map(|(src_entity, dst_entity, _, _)| {
                [
                    src_entity.tlabel().to_string(),
                    dst_entity.tlabel().to_string(),
                ]
            })
            .collect::<HashSet<_>>() // dedup
            .into_iter()
            .collect()
    }

    fn get_vertex_primary_keys(&self) -> HashMap<String, String> {
        self.topologies
            .iter()
            .flat_map(|(src, dst, _, _)| {
                [
                    (
                        src.tlabel().to_owned(),
                        src.primary_key().unwrap().to_owned(),
                    ),
                    (
                        dst.tlabel().to_owned(),
                        dst.primary_key().unwrap().to_owned(),
                    ),
                ]
            })
            .collect()
    }
}

impl TopologyFeatureViewInfo {
    pub(super) fn new(
        infra_manager: &InfraManager,
        topo_tuples: Vec<(Entity, Entity, Entity, Topology)>,
        topo_view: TopologyFeatureView,
    ) -> Self {
        // check infra
        // now assume all topology data in the same graph dataset are in the same database identified by InfraIdentifier
        let mut topo_infra = None;
        topo_tuples.iter().for_each(|(_, _, _, topo)| {
            if topo_infra.is_some() {
                assert!(topo.sink_infra_id == topo_infra);
            } else {
                topo_infra = topo.sink_infra_id.clone();
            }
        });
        let infra_info = match topo_infra.expect("Topology must provide infra info") {
            InfraIdentifier::Neo4j(id) => match infra_manager
                .get_infra_config(&InfraIdentifier::Neo4j(id.clone()))
                .unwrap_or_else(|| panic!("Cannot get infra config {id}"))
            {
                InfraConfig::Neo4jDatabaseProviderConfig {
                    uri,
                    username,
                    password,
                } => HashMap::from([
                    ("infra_type".to_owned(), "neo4j".to_owned()),
                    (
                        "uri".to_owned(),
                        if uri.starts_with("bolt://") {
                            uri.to_owned()
                        } else {
                            format!("bolt://{uri}")
                        },
                    ),
                    ("username".to_owned(), username.to_owned()),
                    ("password".to_owned(), password.to_owned()),
                ]),
                _ => panic!("Expected Neo4jDatabaseProviderConfig"),
            },
            other => {
                panic!("{other:?} infra does not support topo serving")
            }
        };
        Self {
            topologies: topo_tuples,
            infra_info,
            rendering_opt: topo_view.rendering_opt,
        }
    }
}
