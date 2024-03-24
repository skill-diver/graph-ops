use super::GenericResponse;
use ofnil::{FeatureStore, InfraIdentifier};
use rocket::{get, info, response::status::Custom, serde::json::Json, State};

type InfraInfo = (InfraIdentifier, String);

#[get("/infras")]
pub fn get_infras_handler(
    fs: &State<FeatureStore>,
) -> Result<Json<Vec<InfraInfo>>, Custom<Json<GenericResponse>>> {
    info!("Getting all infras");
    Ok(Json(fs.infra_manager().get_infra_info()))
}
