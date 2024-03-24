use std::collections::HashMap;

use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore, Variant};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};

#[get("/field?<id>")]
pub async fn get_field_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::Field>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting field {}", id);
            if let Ok(field) = fs.registry().get_field(&id).await {
                Ok(Json(field))
            } else {
                Err(generate_error_response(format!("Field {id} not found")))
            }
        }
        None => {
            info!("Field ID not provided");
            Err(generate_error_response("Field ID not provided".to_string()))
        }
    }
}

// Note: here is entity name instead of entity id
#[get("/fields?<entity_name>")]
pub async fn get_fields_handler(
    fs: &State<FeatureStore>,
    entity_name: Option<String>,
) -> Result<Json<HashMap<String, ofnil::Field>>, Custom<Json<GenericResponse>>> {
    match entity_name {
        Some(entity_name) => {
            info!("Getting all fields for entity {}", entity_name);
            match fs
                .registry()
                .get_entity_fields(entity_name.as_str(), &Variant::default())
                .await
            {
                Ok(fields) => {
                    let mut fields_map = HashMap::new();
                    for field in fields {
                        fields_map.insert(field.resource_id(), field);
                    }
                    Ok(Json(fields_map))
                }
                Err(e) => Err(generate_error_response(format!(
                    "Error getting fields: {e}",
                ))),
            }
        }
        None => {
            info!("Getting all fields");
            match fs.registry().get_all_fields().await {
                Ok(fields) => {
                    let mut fields_map = HashMap::new();
                    for field in fields {
                        fields_map.insert(field.resource_id(), field);
                    }
                    Ok(Json(fields_map))
                }
                Err(e) => Err(generate_error_response(format!(
                    "Error getting fields: {e}",
                ))),
            }
        }
    }
}

#[post("/field", data = "<field>")]
pub async fn post_field_handler(
    fs: &State<FeatureStore>,
    field: Json<ofnil::Field>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating field {:?}", field);
    match fs
        .registry()
        .register_resource(&(field.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(field.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating field: {e}",
        ))),
    }
}
