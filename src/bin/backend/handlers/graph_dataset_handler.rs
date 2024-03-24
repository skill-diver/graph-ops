use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/graph_dataset?<id>")]
pub async fn get_graph_dataset_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::GraphDataset>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting graph_dataset {}", id);
            if let Ok(graph_dataset) = fs.registry().get_graph_dataset(&id).await {
                Ok(Json(graph_dataset))
            } else {
                Err(generate_error_response(format!(
                    "GraphDataset {id} not found",
                )))
            }
        }
        None => {
            info!("GraphDataset ID not provided");
            Err(generate_error_response(
                "GraphDataset ID not provided".to_string(),
            ))
        }
    }
}

#[get("/graph_datasets")]
pub async fn get_graph_datasets_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::GraphDataset>>, Custom<Json<GenericResponse>>> {
    info!("Getting all graph_datasets");
    match fs.registry().get_all_graph_datasets().await {
        Ok(graph_datasets) => {
            let mut graph_datasets_map = HashMap::new();
            for graph_dataset in graph_datasets {
                graph_datasets_map.insert(graph_dataset.resource_id(), graph_dataset);
            }
            Ok(Json(graph_datasets_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting graph_datasets: {e}",
        ))),
    }
}

#[post("/graph_dataset", data = "<graph_dataset>")]
pub async fn post_graph_dataset_handler(
    fs: &State<FeatureStore>,
    graph_dataset: Json<ofnil::GraphDataset>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating graph_dataset {:?}", graph_dataset);
    match fs
        .registry()
        .register_resource(&(graph_dataset.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(graph_dataset.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating graph_dataset: {e}",
        ))),
    }
}
