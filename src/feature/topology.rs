use chrono::serde::ts_seconds_option;
use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::InfraIdentifier;

use super::ResourceId;
use super::ResourceOp;
use super::Variant;

#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TopologyType {
    AdjacencyList,
    AdjacencyMatrix,
    BipartiteGraphChain,
}

/// Pure graph topology data. Imported from source data files, or extracted from Graphs in the native graph database.
#[pyclass(module = "ofnil")]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Topology {
    #[pyo3(get)]
    pub name: String,
    pub transformation_id: Option<String>,
    #[pyo3(get)]
    pub topology_type: Option<TopologyType>, // None if in native graph database for now
    #[pyo3(get)]
    pub edge_entity_id: Option<ResourceId>, // required for registry
    #[pyo3(get)]
    pub src_node_entity_id: Option<ResourceId>, // required for registry
    #[pyo3(get)]
    pub dst_node_entity_id: Option<ResourceId>, // required for registry
    #[pyo3(get)]
    pub variant: Variant,
    pub description: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    pub tags: HashMap<String, String>,
    pub owners: Vec<String>,

    pub sink_infra_id: Option<InfraIdentifier>,
}

impl ResourceOp for Topology {
    fn resource_id(&self) -> ResourceId {
        format!("{}/{}/{}", &self.variant, "Topology", &self.name)
    }

    fn sink_infra_id(&self) -> Option<InfraIdentifier> {
        self.sink_infra_id.clone()
    }

    fn transformation_id(&self) -> Option<ResourceId> {
        self.transformation_id.clone()
    }
}
