mod storage;

use futures::future::join_all;
use log::{error, info};

use crate::{
    feature::{ResourceId, ResourceOp, Transformation},
    *,
};
use std::error::Error;
use storage::EtcdStorage;

pub struct FeatureRegistry {
    storage: EtcdStorage,
}

impl FeatureRegistry {
    pub async fn new(endpoints: Vec<impl Into<String>>) -> Result<Self, Box<dyn Error>> {
        let storage = EtcdStorage::new(endpoints).await?;
        Ok(Self { storage })
    }

    pub async fn default() -> Result<Self, Box<dyn Error>> {
        FeatureRegistry::new(vec!["http://localhost:2379"]).await
    }

    pub async fn register_resource(
        &self,
        resource: &impl ResourceOp,
    ) -> Result<(), Box<dyn Error>> {
        let key = resource.resource_id();
        let value = serde_json::to_string(&resource)?;
        info!("Registering resource: {} -> {}", &key, &value);
        self.storage.put(&key, &value).await?;
        Ok(())
    }

    pub async fn register_resources(
        &self,
        resources: &Vec<&impl ResourceOp>,
    ) -> Result<(), Box<dyn Error>> {
        for &resource in resources {
            self.register_resource(resource).await?;
        }
        Ok(())
    }

    pub async fn get_string(&self, id: &ResourceId) -> Result<String, Box<dyn Error>> {
        self.storage.get(id).await
    }

    pub async fn get<T>(&self, id: &ResourceId) -> Result<T, Box<dyn Error>>
    where
        T: ResourceOp,
    {
        let value = self.storage.get(id).await?;
        Ok(serde_json::from_str::<T>(&value)?)
    }

    pub async fn get_all(&self, prefix: &str) -> Result<Vec<String>, Box<dyn Error>> {
        self.storage.get_all(prefix).await
    }

    pub async fn get_entity_fields(
        &self,
        entity_name: &str,
        variant: &Variant,
    ) -> Result<Vec<Field>, Box<dyn Error>> {
        let prefix = &format!("{variant}/Field/{entity_name}");
        let values: Result<Vec<Field>, serde_json::Error> = self
            .storage
            .get_all(prefix)
            .await?
            .iter()
            .map(|jstr| serde_json::from_str::<Field>(jstr))
            .collect();

        Ok(
            values.map_err(|e| RegistryError::EntityFieldsRetrievalError {
                entity: entity_name.to_string(),
                variant: Variant::default(),
                super_error: Box::new(e),
            })?,
        )
    }

    pub async fn get_entity(&self, entity_id: &ResourceId) -> Result<Entity, Box<dyn Error>> {
        let value = self.storage.get(entity_id).await?;
        let entity = serde_json::from_str::<Entity>(&value)?;
        Ok(entity)
    }

    pub async fn get_entities(
        &self,
        entity_ids: Vec<&ResourceId>,
    ) -> Result<Vec<Entity>, Box<dyn Error>> {
        let res: Vec<_> = join_all(
            entity_ids
                .iter()
                .map(|entity_id| self.storage.get(entity_id)),
        )
        .await;
        let mut entities = Vec::new();
        for value_err in res {
            entities.push(serde_json::from_str::<Entity>(&value_err?)?)
        }
        Ok(entities)
    }

    pub async fn get_all_entities(&self) -> Result<Vec<Entity>, Box<dyn Error>> {
        let values = self.storage.get_all("default/Entity/").await?; // TODO: configurable variant?
        let mut entities = Vec::new();
        for value in values {
            entities.push(serde_json::from_str::<Entity>(&value)?)
        }
        Ok(entities)
    }

    pub async fn get_field(&self, field_id: &ResourceId) -> Result<Field, Box<dyn Error>> {
        let value = self.storage.get(field_id).await?;
        let field = serde_json::from_str::<Field>(&value)?;
        Ok(field)
    }

    pub async fn get_all_fields(&self) -> Result<Vec<Field>, Box<dyn Error>> {
        let values = self.storage.get_all("default/Field/").await?; // TODO: configurable variant?
        let mut fields = Vec::new();
        for value in values {
            fields.push(serde_json::from_str::<Field>(&value)?)
        }
        Ok(fields)
    }

    pub async fn get_topology(&self, id: &ResourceId) -> Result<Topology, Box<dyn Error>> {
        Ok(serde_json::from_str::<Topology>(
            &self.storage.get(id).await?,
        )?)
    }

    pub async fn get_all_topologies(&self) -> Result<Vec<Topology>, Box<dyn Error>> {
        let values = self.storage.get_all("default/Topology/").await?;
        let mut topologies = Vec::new();
        for value in values {
            topologies.push(serde_json::from_str::<Topology>(&value)?)
        }
        Ok(topologies)
    }

    pub async fn get_table_feature_view(
        &self,
        table_feature_view_id: &ResourceId,
    ) -> Result<TableFeatureView, Box<dyn Error>> {
        let value = self.storage.get(table_feature_view_id).await?;
        Ok(serde_json::from_str::<TableFeatureView>(&value)?)
    }

    pub async fn get_all_table_feature_views(
        &self,
    ) -> Result<Vec<TableFeatureView>, Box<dyn Error>> {
        let values = self.storage.get_all("default/TableFeatureView/").await?;
        let mut table_feature_views = Vec::new();
        for value in values {
            table_feature_views.push(serde_json::from_str::<TableFeatureView>(&value)?)
        }
        Ok(table_feature_views)
    }

    pub async fn get_topology_feature_view(
        &self,
        view_id: &ResourceId,
    ) -> Result<TopologyFeatureView, Box<dyn Error>> {
        let value = self.storage.get(view_id).await?;
        Ok(serde_json::from_str::<TopologyFeatureView>(&value)?)
    }

    pub async fn get_all_topology_feature_views(
        &self,
    ) -> Result<Vec<TopologyFeatureView>, Box<dyn Error>> {
        let values = self.storage.get_all("default/TopologyFeatureView/").await?;
        let mut topology_feature_views = Vec::new();
        for value in values {
            topology_feature_views.push(serde_json::from_str::<TopologyFeatureView>(&value)?)
        }
        Ok(topology_feature_views)
    }

    pub async fn get_transformation(
        &self,
        transformation_id: &ResourceId,
    ) -> Result<Transformation, Box<dyn Error>> {
        let value = self.storage.get(transformation_id).await?;
        let transformation = serde_json::from_str::<Transformation>(&value)?;
        Ok(transformation)
    }

    pub async fn get_all_transformations(&self) -> Result<Vec<Transformation>, Box<dyn Error>> {
        let values = self.storage.get_all("default/Transformation/").await?;
        let mut transformations = Vec::new();
        for value in values {
            transformations.push(serde_json::from_str::<Transformation>(&value)?)
        }
        Ok(transformations)
    }

    pub async fn get_graph(&self, graph_id: &ResourceId) -> Result<Graph, Box<dyn Error>> {
        let value = self.storage.get(graph_id).await?;
        let graph = serde_json::from_str::<Graph>(&value)?;
        Ok(graph)
    }

    pub async fn get_all_graphs(&self) -> Result<Vec<Graph>, Box<dyn Error>> {
        let values = self.storage.get_all("default/Graph/").await?;
        let mut graphs = Vec::new();
        for value in values {
            graphs.push(serde_json::from_str::<Graph>(&value)?)
        }
        Ok(graphs)
    }

    pub async fn get_graph_dataset(
        &self,
        dataset_id: &ResourceId,
    ) -> Result<GraphDataset, Box<dyn Error>> {
        let value = self.storage.get(dataset_id).await?;
        let dataset = serde_json::from_str::<GraphDataset>(&value)?;
        Ok(dataset)
    }

    pub async fn get_all_graph_datasets(&self) -> Result<Vec<GraphDataset>, Box<dyn Error>> {
        let values = self.storage.get_all("default/GraphDataset/").await?;
        let mut datasets = Vec::new();
        for value in values {
            datasets.push(serde_json::from_str::<GraphDataset>(&value)?)
        }
        Ok(datasets)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RegistryError {
    #[error("Error retrieving fields associated with entity {entity} with version {variant}.\nOriginal error: {super_error}")]
    EntityFieldsRetrievalError {
        entity: String,
        variant: Variant,
        super_error: Box<dyn Error>,
    },
}
