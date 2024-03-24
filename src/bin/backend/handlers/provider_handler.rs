use super::{generate_error_response, GenericResponse};
use ofnil::{infra::pi::SchemaProvider, FeatureStore};
use rocket::{get, post, response::status::Custom, serde::json::Json, State};

#[get("/provider/entities?<infra_name>")]
pub async fn get_entities_provider_handler(
    fs: &State<FeatureStore>,
    infra_name: Option<String>,
) -> Result<Json<Vec<ofnil::Entity>>, Custom<Json<GenericResponse>>> {
    return match fs
        .infra_manager()
        .get_neo4j_connector(&infra_name.unwrap_or("neo4j_1".to_string()))
    {
        Some(connector) => Ok(Json(
            connector.get_database().get_all_entities().await.unwrap(),
        )),
        None => Err(generate_error_response(
            "Connector is not found".to_string(),
        )),
    };
}

#[post("/provider/fields?<infra_name>", data = "<entity>")]
pub async fn get_fields_via_post_entity_provider_handler(
    fs: &State<FeatureStore>,
    infra_name: Option<String>,
    entity: Json<ofnil::Entity>,
) -> Result<Json<Vec<ofnil::Field>>, Custom<Json<GenericResponse>>> {
    return match fs
        .infra_manager()
        .get_neo4j_connector(&infra_name.unwrap_or("neo4j_1".to_string()))
    {
        Some(connector) => Ok(Json(
            connector.get_database().get_fields(&entity).await.unwrap(),
        )),
        None => Err(generate_error_response(
            "Connector is not found".to_string(),
        )),
    };
}
