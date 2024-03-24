use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::InfraIdentifier;
use crate::Topology;

use super::ResourceId;
use super::ResourceOp;
use super::{Entity, Variant};

#[pyclass(get_all)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graph {
    pub name: String,
    pub variant: Variant,
    pub description: Option<String>,
    pub entity_ids: HashMap<String, ResourceId>,
    pub tags: HashMap<String, String>,
    pub owners: Vec<String>,
    pub sink_infra_id: Option<InfraIdentifier>,
}

impl ResourceOp for Graph {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", &self.variant, "Graph", &self.name)
    }
}

impl Graph {
    pub fn new(
        name: &str,
        variant: Variant,
        entities: Vec<&Entity>,
        sink_infra_id: Option<InfraIdentifier>,
    ) -> Self {
        Graph {
            name: name.to_string(),
            variant,
            description: None,
            entity_ids: entities
                .into_iter()
                .map(|e| (e.tlabel().to_owned(), e.resource_id()))
                .collect(),
            tags: HashMap::new(),
            owners: Vec::new(),
            sink_infra_id,
        }
    }

    pub fn project_topology(&self, edge_types: Vec<&str>) -> Vec<Topology> {
        let mut res = Vec::new();
        for etype in edge_types {
            if let Some(edge_entity_id) = self.entity_ids.get(etype) {
                let mut split = edge_entity_id.split('/');
                let variant = split.next().unwrap();
                res.push(Topology {
                    name: format!("{}_{}", self.name, etype),
                    transformation_id: None,
                    topology_type: None,
                    sink_infra_id: self.sink_infra_id.clone(),
                    edge_entity_id: Some(edge_entity_id.clone()),
                    src_node_entity_id: Some(format!(
                        "{}/Entity/{}",
                        variant,
                        split.nth(2).unwrap()
                    )),
                    dst_node_entity_id: Some(format!(
                        "{}/Entity/{}",
                        variant,
                        split.next().unwrap()
                    )),
                    ..Default::default()
                });
            }
        }
        res
    }
}
