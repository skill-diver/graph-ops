use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/transformation?<id>")]
pub async fn get_transformation_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::Transformation>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting transformation {}", id);
            if let Ok(transformation) = fs.registry().get_transformation(&id).await {
                Ok(Json(transformation))
            } else {
                Err(generate_error_response(format!(
                    "Transformation {id} not found",
                )))
            }
        }
        None => {
            info!("Transformation ID not provided");
            Err(generate_error_response(
                "Transformation ID not provided".to_string(),
            ))
        }
    }
}

#[get("/transformations")]
pub async fn get_transformations_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::Transformation>>, Custom<Json<GenericResponse>>> {
    info!("Getting all transformations");
    match fs.registry().get_all_transformations().await {
        Ok(transformations) => {
            let mut transformations_map = HashMap::new();
            for transformation in transformations {
                transformations_map.insert(transformation.resource_id(), transformation);
            }
            Ok(Json(transformations_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting transformations: {e}",
        ))),
    }
}

#[post("/transformation", data = "<transformation>")]
pub async fn post_transformation_handler(
    fs: &State<FeatureStore>,
    transformation: Json<ofnil::Transformation>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating transformation {:?}", transformation);
    match fs
        .registry()
        .register_resource(&(transformation.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(transformation.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating transformation: {e}",
        ))),
    }
}
