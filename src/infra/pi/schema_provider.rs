use std::error::Error;

use crate::{Entity, FeatureRegistry, Field, Graph, InfraIdentifier};

#[async_trait::async_trait]
pub trait SchemaProvider {
    async fn get_node_field_resource(&self) -> Result<(), Box<dyn Error>>;

    async fn get_rel_field_resource(&self) -> Result<(), Box<dyn Error>>;

    async fn get_all_entities(&self) -> Result<Vec<Entity>, Box<dyn Error>>;

    async fn get_fields(&self, entity: &Entity) -> Result<Vec<Field>, Box<dyn Error>>;

    fn get_infra_id(&self) -> Option<InfraIdentifier>;

    async fn register_graph(&self, registry: &FeatureRegistry) -> Result<Graph, Box<dyn Error>>;
}
