use super::GenericResponse;
use ofnil::infra::pi::GAF;
use rocket::{get, info, response::status::Custom, serde::json::Json};
use strum::IntoEnumIterator;

#[get("/gaf")]
pub async fn get_gaf_handler() -> Result<Json<Vec<String>>, Custom<Json<GenericResponse>>> {
    info!("Getting all GAFs");
    let gafs = GAF::iter().map(|f| f.to_string()).collect::<Vec<_>>();
    Ok(Json(gafs))
}
