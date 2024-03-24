use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/topology?<id>")]
pub async fn get_topology_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::Topology>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting topology {}", id);
            if let Ok(topology) = fs.registry().get_topology(&id).await {
                Ok(Json(topology))
            } else {
                Err(generate_error_response(format!("Topology {id} not found")))
            }
        }
        None => {
            info!("Topology ID not provided");
            Err(generate_error_response(
                "Topology ID not provided".to_string(),
            ))
        }
    }
}

#[get("/topologies")]
pub async fn get_topologies_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::Topology>>, Custom<Json<GenericResponse>>> {
    info!("Getting all topologies");
    match fs.registry().get_all_topologies().await {
        Ok(topologies) => {
            let mut topologies_map = HashMap::new();
            for topology in topologies {
                topologies_map.insert(topology.resource_id(), topology);
            }
            Ok(Json(topologies_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting topologies: {e}",
        ))),
    }
}

#[post("/topology", data = "<topology>")]
pub async fn post_topology_handler(
    fs: &State<FeatureStore>,
    topology: Json<ofnil::Topology>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating topology {:?}", topology);
    match fs
        .registry()
        .register_resource(&(topology.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(topology.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating topology: {e}",
        ))),
    }
}
