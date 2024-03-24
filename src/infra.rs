//! This modules specifies interfaces for connecting external infras and provides an [InfraManager]
//! to manage all infrastructure in the project. [Connectors](connectors) implement the interfaces
//! to provide storage and/or transformation functionalities.

pub mod connectors;
pub mod pi;

use connectors::*;
use pi::{StorageConnector, TransformationConnector};

use log::debug;
use pyo3::{exceptions::PyValueError, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

use crate::{config::InfraConfig, SchemaProvider};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InfraIdentifier {
    Neo4j(String),
    Redis(String),
}

impl IntoPy<PyObject> for InfraIdentifier {
    fn into_py(self, py: Python<'_>) -> PyObject {
        serde_json::from_value::<HashMap<String, String>>(serde_json::to_value(self).unwrap())
            .unwrap()
            .into_py(py)
    }
}

impl FromPyObject<'_> for InfraIdentifier {
    fn extract(ob: &'_ PyAny) -> PyResult<Self> {
        let infra_id = ob.extract::<HashMap<String, String>>()?;
        serde_json::from_value(json!(infra_id)).map_err(|e| {
            PyValueError::new_err(format!(
                "Cannot parse infra identifier from {infra_id:?}. {e}"
            ))
        })
    }
}

#[derive(Debug)]
pub enum Infra {
    Neo4j(Neo4jConnector, Option<InfraConfig>),
    Redis(RedisConnector, Option<InfraConfig>),
}

impl Infra {
    pub fn get_uri(&self) -> String {
        let config = match &self {
            Infra::Neo4j(_, config) => config.as_ref().unwrap(),
            Infra::Redis(_, config) => config.as_ref().unwrap(),
        };
        match config {
            InfraConfig::Neo4jDatabaseProviderConfig { uri, .. } => uri.clone(),
            InfraConfig::RedisClientConfig { uri } => uri.clone(),
        }
    }
}

#[derive(Default)]
pub struct InfraManager {
    pub infras: HashMap<InfraIdentifier, Infra>,
}

impl InfraManager {
    fn new() -> Self {
        Self {
            infras: HashMap::new(),
        }
    }

    pub(crate) async fn from_config(config: &HashMap<String, InfraConfig>) -> Self {
        let mut infras = InfraManager::new();
        for (name, infra) in config {
            debug!("Creating infra: {} {:?}", name, infra);
            match infra {
                InfraConfig::Neo4jDatabaseProviderConfig {
                    uri,
                    username,
                    password,
                } => {
                    infras.register_neo4j_connector(
                        name,
                        Neo4jConnector::new(
                            uri.to_string(),
                            username.to_string(),
                            password.to_string(),
                            None,
                            Some(InfraIdentifier::Neo4j(name.to_owned())),
                        )
                        .await
                        .unwrap(),
                        Some(infra.clone()),
                    );
                }
                InfraConfig::RedisClientConfig { uri } => {
                    infras.register_redis_connector(
                        name,
                        RedisConnector::new(uri.to_string()),
                        Some(infra.clone()),
                    );
                }
            }
        }

        infras
    }

    #[inline]
    pub fn add_infra(&mut self, infra_id: InfraIdentifier, infra: Infra) {
        debug!("Adding infra: {infra_id:?}");
        self.infras.insert(infra_id, infra);
    }

    #[inline]
    fn get_infra(&self, id: &InfraIdentifier) -> Option<&Infra> {
        self.infras.get(id)
    }

    pub fn get_infra_config(&self, infra_id: &InfraIdentifier) -> Option<&InfraConfig> {
        self.infras.get(infra_id).map(|infra| match infra {
            Infra::Neo4j(_, Some(conf)) => conf,
            Infra::Redis(_, Some(conf)) => conf,
            _ => panic!("Cannot get conf"),
        })
    }

    pub fn get_storage_infra(&self, infra_id: &InfraIdentifier) -> Option<&dyn StorageConnector> {
        match self.get_infra(infra_id) {
            Some(Infra::Redis(connector, _)) => Some(connector),
            Some(Infra::Neo4j(connector, _)) => Some(connector),
            _ => None,
        }
    }

    pub fn get_storage_infra_cloned(
        &self,
        infra_id: &InfraIdentifier,
    ) -> Option<Box<dyn StorageConnector>> {
        match self.get_infra(infra_id) {
            Some(Infra::Redis(connector, _)) => Some(Box::new(connector.clone())),
            Some(Infra::Neo4j(connector, _)) => Some(Box::new(connector.clone())),
            _ => None,
        }
    }

    #[cfg(feature = "dashboard")]
    pub fn get_graph_transformation_infra_ids(&self) -> Vec<InfraIdentifier> {
        self.infras
            .keys()
            .filter_map(|id| match id {
                InfraIdentifier::Neo4j(_) => Some(id.clone()),
                InfraIdentifier::Redis(_) => None,
            })
            .collect()
    }

    pub fn get_graph_transformation_infra(
        &self,
        infra_id: &InfraIdentifier,
    ) -> Option<&dyn TransformationConnector> {
        match self.get_infra(infra_id) {
            Some(Infra::Neo4j(connector, _)) => Some(connector),
            _ => None,
        }
    }

    pub fn get_graph_transformation_infra_cloned(
        &self,
        infra_id: &InfraIdentifier,
    ) -> Option<Box<dyn TransformationConnector>> {
        match self.get_infra(infra_id) {
            Some(Infra::Neo4j(connector, _)) => Some(Box::new(connector.clone())),
            _ => None,
        }
    }

    pub fn register_neo4j_connector(
        &mut self,
        infra_id_name: impl Into<String>,
        neo4j_connector: Neo4jConnector,
        config: Option<InfraConfig>,
    ) {
        self.add_infra(
            InfraIdentifier::Neo4j(infra_id_name.into()),
            Infra::Neo4j(neo4j_connector, config),
        );
    }

    pub fn get_neo4j_connector(&self, infra_id_name: impl Into<String>) -> Option<&Neo4jConnector> {
        match self.get_infra(&InfraIdentifier::Neo4j(infra_id_name.into())) {
            Some(Infra::Neo4j(connector, _)) => Some(connector),
            _ => None,
        }
    }

    pub fn register_redis_connector(
        &mut self,
        infra_id_name: impl Into<String>,
        connector: RedisConnector,
        config: Option<InfraConfig>,
    ) {
        self.add_infra(
            InfraIdentifier::Redis(infra_id_name.into()),
            Infra::Redis(connector, config),
        );
    }

    pub fn get_redis_connector(&self, infra_id_name: impl Into<String>) -> Option<&RedisConnector> {
        match self.get_infra(&InfraIdentifier::Redis(infra_id_name.into())) {
            Some(Infra::Redis(connector, _)) => Some(connector),
            _ => None,
        }
    }

    pub fn get_infra_info(&self) -> Vec<(InfraIdentifier, String)> {
        self.infras
            .iter()
            .map(|(key, value)| (key.clone(), value.get_uri()))
            .collect()
    }

    pub fn get_schema_provider(&self, infra_id: &InfraIdentifier) -> Arc<dyn SchemaProvider> {
        match infra_id {
            InfraIdentifier::Neo4j(neo4j_id) => {
                self.get_neo4j_connector(neo4j_id).unwrap().get_database()
            }
            InfraIdentifier::Redis(_) => {
                // TODO(Pond): create schema provider for redis
                panic!("Not implemented");
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_infra_manager_bolt_conn() {
    use bb8_bolt::bolt_client::{bolt_proto::message::Success, Metadata};

    let mut infra_manager = InfraManager::default();

    infra_manager.register_neo4j_connector(
        "neo4j_1".to_string(),
        Neo4jConnector::new(
            "localhost:7687",
            "neo4j",
            "ofnil",
            None,
            Some(InfraIdentifier::Neo4j("neo4j_1".to_string())),
        )
        .await
        .unwrap(),
        None,
    );

    let neo4j_provider = infra_manager
        .get_neo4j_connector(&"neo4j_1".to_string())
        .unwrap()
        .get_database();

    let mut bolt_conn = neo4j_provider.get_bolt_connection().await.unwrap();

    let msg = bolt_conn
        .run("MATCH (n) RETURN n LIMIT 1", None, None)
        .await
        .unwrap();
    assert!(Success::try_from(msg).is_ok(), "MATCH failure");

    let (records, msg) = bolt_conn
        .pull(Some(Metadata::from_iter(vec![("n", 1)])))
        .await
        .unwrap();
    assert!(Success::try_from(msg).is_ok(), "PULL failure");
    println!("records: {records:?}");
}
