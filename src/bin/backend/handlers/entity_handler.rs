use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/entity?<id>")]
pub async fn get_entity_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::Entity>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting entity {}", id);
            if let Ok(entity) = fs.registry().get_entity(&id).await {
                Ok(Json(entity))
            } else {
                Err(generate_error_response(format!("Entity {id} not found")))
            }
        }
        None => {
            info!("Entity ID not provided");
            Err(generate_error_response(
                "Entity ID not provided".to_string(),
            ))
        }
    }
}

#[get("/entities")]
pub async fn get_entities_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::Entity>>, Custom<Json<GenericResponse>>> {
    info!("Getting all entities");
    match fs.registry().get_all_entities().await {
        Ok(entities) => {
            let mut entities_map = HashMap::new();
            for entity in entities {
                entities_map.insert(entity.resource_id(), entity);
            }
            Ok(Json(entities_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting entities: {e}",
        ))),
    }
}

#[post("/entity", data = "<entity>")]
pub async fn post_entity_handler(
    fs: &State<FeatureStore>,
    entity: Json<ofnil::Entity>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating entity {:?}", entity);
    match fs
        .registry()
        .register_resource(&(entity.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(entity.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating entity: {e}",
        ))),
    }
}
