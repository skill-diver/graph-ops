use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/table_feature_view?<id>")]
pub async fn get_table_feature_view_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::TableFeatureView>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting table_feature_view {}", id);
            if let Ok(table_feature_view) = fs.registry().get_table_feature_view(&id).await {
                Ok(Json(table_feature_view))
            } else {
                Err(generate_error_response(format!(
                    "TableFeatureView {id} not found",
                )))
            }
        }
        None => {
            info!("TableFeatureView ID not provided");
            Err(generate_error_response(
                "TableFeatureView ID not provided".to_string(),
            ))
        }
    }
}

#[get("/table_feature_views")]
pub async fn get_table_feature_views_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::TableFeatureView>>, Custom<Json<GenericResponse>>> {
    info!("Getting all table_feature_views");
    match fs.registry().get_all_table_feature_views().await {
        Ok(table_feature_views) => {
            let mut table_feature_views_map = HashMap::new();
            for table_feature_view in table_feature_views {
                table_feature_views_map
                    .insert(table_feature_view.resource_id(), table_feature_view);
            }
            Ok(Json(table_feature_views_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting table_feature_views: {e}",
        ))),
    }
}

#[post("/table_feature_view", data = "<table_feature_view>")]
pub async fn post_table_feature_view_handler(
    fs: &State<FeatureStore>,
    table_feature_view: Json<ofnil::TableFeatureView>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating table_feature_view {:?}", table_feature_view);
    match fs
        .registry()
        .register_resource(&(table_feature_view.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(table_feature_view.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating table_feature_view: {e}",
        ))),
    }
}
