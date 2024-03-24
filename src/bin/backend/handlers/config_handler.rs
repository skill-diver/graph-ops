use super::{generate_error_response, GenericResponse};
use ofnil::{transformation::BuiltInFnArgs, FeatureStore};
use rocket::{get, info, response::status::Custom, serde::json::Json, State};
use serde::Serialize;
use serde_json::{json, Value};
use std::str::FromStr;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Select,
    Multiple,
    Multiselect,
}

#[derive(Serialize)]
pub struct ConfigItem {
    input_type: InputType,
    key: String,
    value: Value,
}

impl ConfigItem {
    fn new(input_type: InputType, key: impl Into<String>, value: Value) -> Self {
        Self {
            input_type,
            key: key.into(),
            value,
        }
    }
}

#[derive(Serialize)]
pub enum Config {
    VertexFeatureTransformation {
        infra: ConfigItem,
        algorithm: BuiltInFnArgs,
        graph_projection: Vec<ConfigItem>, // vertex label, edge label
        output_feature: Vec<ConfigItem>,   // target_vertex_tlabel and output_names
    },
    Export {
        sink_infra: ConfigItem,
    },
}

#[get("/configs/<target>")]
pub async fn get_configs_handler(
    fs: &State<FeatureStore>,
    target: String,
) -> Result<Json<Config>, Custom<Json<GenericResponse>>> {
    info!("Getting configs for {target:?}");
    if target == "export" {
        Ok(Json(Config::Export {
            sink_infra: ConfigItem::new(
                InputType::Select,
                "infra",
                json!(fs
                    .infra_manager()
                    .get_infra_info()
                    .into_iter()
                    .map(|(id, _)| { id })
                    .collect::<Vec<_>>()),
            ),
        }))
    } else {
        match BuiltInFnArgs::from_str(&target) {
            Ok(args) => {
                // FIXME(tatiana): retrieve from registry, now hardcode
                let vertex_labels = vec!["Reviewer", "Product"];
                let edge_labels = vec!["alsoView", "alsoBuy", "isSimilarTo", "sameRates", "rates"];
                Ok(Json(Config::VertexFeatureTransformation {
                    infra: ConfigItem::new(
                        InputType::Select,
                        "infra",
                        json!(fs.infra_manager().get_graph_transformation_infra_ids()),
                    ),
                    algorithm: args,
                    graph_projection: vec![
                        ConfigItem::new(InputType::Multiselect, "vertices", json!(vertex_labels)),
                        ConfigItem::new(InputType::Multiselect, "edges", json!(edge_labels)),
                    ],
                    output_feature: vec![
                        ConfigItem::new(InputType::Select, "target_vertex", json!(vertex_labels)),
                        ConfigItem::new(
                            InputType::Multiple,
                            "feature_name(s)",
                            json!(vec![target]),
                        ),
                    ],
                }))
            }
            Err(error) => Err(generate_error_response(error.to_string())),
        }
    }
}
