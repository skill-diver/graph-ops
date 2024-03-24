use futures::future::join_all;
use log::{debug, info};

use crate::{
    config::FeatureStoreConfig,
    feature::{ResourceId, ResourceOp},
    transformation::*,
    *,
};
use std::{collections::HashMap, error::Error, path::Path};

pub struct FeatureStore {
    pub(crate) project: String,
    pub(crate) registry: FeatureRegistry,
    pub(crate) infra_manager: InfraManager,
}

impl FeatureStore {
    fn new(project: String, registry: FeatureRegistry, infra_manager: InfraManager) -> Self {
        Self {
            project,
            registry,
            infra_manager,
        }
    }

    pub async fn default() -> Result<Self, Box<dyn Error>> {
        let project = "Graph Feature Store".to_string();
        let registry = FeatureRegistry::default().await?;
        let infra_manager = InfraManager::default();

        Ok(FeatureStore::new(project, registry, infra_manager))
    }

    pub async fn init(ofnil_home: Option<&str>) -> Result<Self, Box<dyn Error>> {
        if ofnil_home.is_none() && std::env::var("OFNIL_HOME").is_err() {
            return Err(
                "OFNIL_HOME is neither given as funciton parameter nor set in environment variable"
                    .into(),
            );
        }

        let p = std::env::var("OFNIL_HOME").unwrap_or_default();
        let ofnil_home = match ofnil_home {
            Some(path) => Path::new(path),
            None => Path::new(p.as_str()),
        };

        let config = FeatureStoreConfig::from_dir(ofnil_home).unwrap();
        debug!("Config: {:?}", config);

        FeatureStore::from_config(&config).await
    }

    async fn from_config(config: &FeatureStoreConfig) -> Result<Self, Box<dyn Error>> {
        let project = config.project.clone();
        let registry = FeatureRegistry::new(config.registry_endpoints.clone()).await?;
        let infra_manager = InfraManager::from_config(&config.infra_manager).await;

        Ok(FeatureStore::new(project, registry, infra_manager))
    }

    /// @param resource Resource id of FeatureView or (Graph)Dataset
    pub async fn deploy(&self, resource: ResourceId) -> Result<(), Box<dyn Error>> {
        info!("{}: deploy resource {}", self.project, resource);
        let resource = self.registry.get_string(&resource).await?;
        let transformation_to_data = if let Ok(table_feature_view) =
            serde_json::from_str::<TableFeatureView>(&resource)
        {
            self.get_transformations_of_view_items::<Field>(&table_feature_view.field_ids)
                .await
        } else if let Ok(topo_feature_view) = serde_json::from_str::<TopologyFeatureView>(&resource)
        {
            self.get_transformations_of_view_items::<Topology>(&topo_feature_view.topology_ids)
                .await
        } else {
            let graph_dataset = serde_json::from_str::<GraphDataset>(&resource)?;
            let mut transformation_data = HashMap::new();
            for view in graph_dataset.table_feature_views {
                let res = self
                    .get_transformations_of_view_items::<Field>(&view.field_ids)
                    .await;
                transformation_data.extend(res);
            }
            for view in graph_dataset.topology_feature_views {
                let res = self
                    .get_transformations_of_view_items::<Topology>(&view.topology_ids)
                    .await;
                transformation_data.extend(res);
            }

            transformation_data
        };

        info!("Deploy transformation data {:?}", &transformation_to_data);

        // TODO(tatiana): support inter-transformation dependencies. now we assume all transformations compute on the source data only
        join_all(
            transformation_to_data
                .into_iter()
                .map(|(transformation_id, data_ids)| {
                    self.execute_transformation(transformation_id, data_ids)
                }),
        )
        .await
        .iter()
        .for_each(|res| {
            let res = res.as_ref().unwrap();
            info!("Deployment output {:#?}", res);
        });
        Ok(())
    }

    async fn execute_transformation(
        &self,
        transformation_id: ResourceId,
        data_ids: Vec<ResourceId>,
    ) -> Result<TransformationOutputHandler, Box<dyn Error>> {
        let transformation = self.registry.get_transformation(&transformation_id).await?;
        let tc = serde_json::from_str::<TransformationContext>(&transformation.body)?;
        let mut plan = tc.get_materialization_plan(
            data_ids
                .into_iter()
                .map(|resource_id| transformation.get_data_id(&resource_id))
                .collect(),
        );
        plan.orchestrate_infras(&self.infra_manager);
        plan.execute(&self.infra_manager).await
    }

    async fn get_transformations_of_view_items<T>(
        &self,
        items: &[ResourceId],
    ) -> HashMap<String, Vec<ResourceId>>
    where
        T: ResourceOp,
    {
        let mut transformation_data = HashMap::new();
        join_all(items.iter().map(|id| self.registry.get::<T>(id)))
            .await
            .into_iter()
            .for_each(|res| {
                let res = res.unwrap();
                if let Some(tid) = &res.transformation_id() {
                    transformation_data
                        .entry(tid.to_owned())
                        .or_insert_with(Vec::new)
                        .push(res.resource_id());
                }
            });
        transformation_data
    }

    pub fn registry(&self) -> &FeatureRegistry {
        &self.registry
    }

    pub fn infra_manager(&self) -> &InfraManager {
        &self.infra_manager
    }
}
