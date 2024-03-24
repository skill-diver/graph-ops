use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/topology_feature_view?<id>")]
pub async fn get_topology_feature_view_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::TopologyFeatureView>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting topology_feature_view {}", id);
            if let Ok(topology_feature_view) = fs.registry().get_topology_feature_view(&id).await {
                Ok(Json(topology_feature_view))
            } else {
                Err(generate_error_response(format!(
                    "TopologyFeatureView {id} not found",
                )))
            }
        }
        None => {
            info!("TopologyFeatureView ID not provided");
            Err(generate_error_response(
                "TopologyFeatureView ID not provided".to_string(),
            ))
        }
    }
}

#[get("/topology_feature_views")]
pub async fn get_topology_feature_views_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::TopologyFeatureView>>, Custom<Json<GenericResponse>>> {
    info!("Getting all topology_feature_views");
    match fs.registry().get_all_topology_feature_views().await {
        Ok(topology_feature_views) => {
            let mut topology_feature_views_map = HashMap::new();
            for topology_feature_view in topology_feature_views {
                topology_feature_views_map
                    .insert(topology_feature_view.resource_id(), topology_feature_view);
            }
            Ok(Json(topology_feature_views_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting topology_feature_views: {e}",
        ))),
    }
}

#[post("/topology_feature_view", data = "<topology_feature_view>")]
pub async fn post_topology_feature_view_handler(
    fs: &State<FeatureStore>,
    topology_feature_view: Json<ofnil::TopologyFeatureView>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating topology_feature_view {:?}", topology_feature_view);
    match fs
        .registry()
        .register_resource(&(topology_feature_view.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(topology_feature_view.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating topology_feature_view: {e}",
        ))),
    }
}
