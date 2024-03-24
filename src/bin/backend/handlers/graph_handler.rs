use super::{generate_error_response, GenericResponse};
use ofnil::{feature::ResourceOp, FeatureStore};
use rocket::{get, info, post, response::status::Custom, serde::json::Json, State};
use std::collections::HashMap;

#[get("/graph?<id>")]
pub async fn get_graph_handler(
    fs: &State<FeatureStore>,
    id: Option<String>,
) -> Result<Json<ofnil::Graph>, Custom<Json<GenericResponse>>> {
    match id {
        Some(id) => {
            info!("Getting graph {}", id);
            if let Ok(graph) = fs.registry().get_graph(&id).await {
                Ok(Json(graph))
            } else {
                Err(generate_error_response(format!("Graph {id} not found")))
            }
        }
        None => {
            info!("Graph ID not provided");
            Err(generate_error_response("Graph ID not provided".to_string()))
        }
    }
}

#[get("/graphs")]
pub async fn get_graphs_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<HashMap<String, ofnil::Graph>>, Custom<Json<GenericResponse>>> {
    info!("Getting all graphs");
    match fs.registry().get_all_graphs().await {
        Ok(graphs) => {
            let mut graphs_map = HashMap::new();
            for graph in graphs {
                graphs_map.insert(graph.resource_id(), graph);
            }
            Ok(Json(graphs_map))
        }
        Err(e) => Err(generate_error_response(format!(
            "Error getting graphs: {e}",
        ))),
    }
}

#[post("/graph", data = "<graph>")]
pub async fn post_graph_handler(
    fs: &State<FeatureStore>,
    graph: Json<ofnil::Graph>,
) -> Result<String, Custom<Json<GenericResponse>>> {
    info!("Creating graph {:?}", graph);
    match fs
        .registry()
        .register_resource(&(graph.clone().into_inner()))
        .await
    {
        Ok(_) => Ok(graph.resource_id()),
        Err(e) => Err(generate_error_response(format!(
            "Error creating graph: {e}",
        ))),
    }
}
