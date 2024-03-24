use super::responses::GenericResponse;

use rocket::{http::Status, response::status::Custom, serde::json::Json};

pub mod config_handler;
pub mod entity_handler;
pub mod field_handler;
pub mod gaf_handler;
pub mod graph_dataset_handler;
pub mod graph_handler;
pub mod infra_handler;
pub mod provider_handler;
pub mod table_feature_view_handler;
pub mod topology_feature_view_handler;
pub mod topology_handler;
pub mod transformation_handler;

fn generate_error_response(message: String) -> Custom<Json<GenericResponse>> {
    let error_response = GenericResponse {
        status: "fail".to_string(),
        message,
    };
    Custom(Status::InternalServerError, Json(error_response))
}
