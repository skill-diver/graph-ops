use std::{collections::HashMap, error::Error, path::Path};

use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FeatureStoreConfig {
    pub(crate) project: String,
    pub(crate) registry_endpoints: Vec<String>,
    pub(crate) infra_manager: HashMap<String, InfraConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InfraConfig {
    Neo4jDatabaseProviderConfig {
        uri: String,
        username: String,
        password: String,
    },
    RedisClientConfig {
        uri: String,
        // TODO(han): add the support for password in connection info
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct RawFeatureStoreConfig {
    project: String,
    registry_endpoints: Vec<String>,
    infra: Vec<RawInfraConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawInfraConfig {
    name: String,
    infra_type: String,
    env_uri: Option<String>,
    env_username: Option<String>,
    env_password: Option<String>,
    uri: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl FeatureStoreConfig {
    pub(crate) fn from_dir(path: &Path) -> Result<Self, Box<dyn Error>> {
        let config_path = path.join("ofnil.toml");
        let dotenv_path = path.join(".env");
        dotenv::from_path(dotenv_path.as_path()).ok();

        let raw_config = read_toml_to_raw_config(config_path.as_path()).unwrap();
        let config = raw_to_config(raw_config);
        debug!("Config: {:?}", config);

        Ok(config)
    }
}

fn read_toml_to_raw_config(filename: &Path) -> Result<RawFeatureStoreConfig, Box<dyn Error>> {
    let config: RawFeatureStoreConfig =
        toml::from_str(std::fs::read_to_string(filename)?.as_str())?;
    Ok(config)
}

fn raw_to_config(raw_config: RawFeatureStoreConfig) -> FeatureStoreConfig {
    let mut infra_manager = HashMap::new();
    for infra in raw_config.infra {
        debug!("Infra Config: {:?}", infra);
        let infra_config = match infra.infra_type.as_str() {
            "neo4j" => InfraConfig::Neo4jDatabaseProviderConfig {
                uri: {
                    let uri = infra.uri.unwrap_or_else(|| {
                        dotenv::var(infra.env_uri.clone().unwrap_or_default())
                            .unwrap_or_else(|_| "".to_string())
                    });

                    // TODO(tatiana): stripping prefix URI scheme can be problematic
                    if uri.starts_with("bolt://") {
                        uri.strip_prefix("bolt://").unwrap().to_string()
                    } else if uri.starts_with("neo4j://") {
                        uri.strip_prefix("neo4j://").unwrap().to_string()
                    } else {
                        uri
                    }
                },
                username: infra.username.unwrap_or_else(|| {
                    dotenv::var(infra.env_username.unwrap_or_default())
                        .unwrap_or_else(|_| "".to_string())
                }),
                password: infra.password.unwrap_or_else(|| {
                    dotenv::var(infra.env_password.unwrap_or_default())
                        .unwrap_or_else(|_| "".to_string())
                }),
            },
            "redis" => InfraConfig::RedisClientConfig {
                uri: {
                    let uri = infra.uri.unwrap_or_else(|| {
                        dotenv::var(infra.env_uri.unwrap_or_default())
                            .unwrap_or_else(|_| "".to_string())
                    });
                    if uri.starts_with("redis://") {
                        uri
                    } else {
                        format!("redis://{uri}")
                    }
                },
            },

            _ => panic!("Unknown infra type"),
        };
        infra_manager.insert(infra.name, infra_config);
    }
    debug!("Infra manager: {:?}", infra_manager);
    FeatureStoreConfig {
        project: raw_config.project,
        registry_endpoints: raw_config.registry_endpoints,
        infra_manager,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_parse_toml() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config =
            read_toml_to_raw_config(dir.join("examples/quickstart/ofnil.toml").as_path()).unwrap();
        println!("{config:?}");
    }

    #[test]
    fn test_raw_to_config() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dotenv::from_path(dir.join("examples/quickstart/.env")).ok();
        let raw_config =
            read_toml_to_raw_config(dir.join("examples/quickstart/ofnil.toml").as_path()).unwrap();
        let config = raw_to_config(raw_config);
        println!("{config:?}");
    }
}
