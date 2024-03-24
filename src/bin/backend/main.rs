use std::path::PathBuf;

use ofnil::FeatureStore;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    launch, routes, Request, Response,
};

mod handlers;
use handlers::{
    config_handler::get_configs_handler,
    entity_handler::{get_entities_handler, get_entity_handler, post_entity_handler},
    field_handler::{get_field_handler, get_fields_handler, post_field_handler},
    gaf_handler::get_gaf_handler,
    graph_dataset_handler::{
        get_graph_dataset_handler, get_graph_datasets_handler, post_graph_dataset_handler,
    },
    graph_handler::{get_graph_handler, get_graphs_handler, post_graph_handler},
    infra_handler::get_infras_handler,
    provider_handler::{
        get_entities_provider_handler, get_fields_via_post_entity_provider_handler,
    },
    table_feature_view_handler::{
        get_table_feature_view_handler, get_table_feature_views_handler,
        post_table_feature_view_handler,
    },
    topology_feature_view_handler::{
        get_topology_feature_view_handler, get_topology_feature_views_handler,
        post_topology_feature_view_handler,
    },
    topology_handler::{get_topologies_handler, get_topology_handler, post_topology_handler},
    transformation_handler::{
        get_transformation_handler, get_transformations_handler, post_transformation_handler,
    },
};

mod responses;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
async fn rocket() -> _ {
    // FIXME: use config or env
    let fs = FeatureStore::init(
        PathBuf::from_iter(vec![env!("CARGO_MANIFEST_DIR"), "examples/quickstart/"]).to_str(),
    )
    .await
    .unwrap();

    rocket::build().manage(fs).attach(CORS).mount(
        "/",
        routes![
            // entity
            get_entity_handler,
            get_entities_handler,
            post_entity_handler,
            // field
            get_field_handler,
            get_fields_handler,
            post_field_handler,
            // transformation
            get_transformation_handler,
            get_transformations_handler,
            post_transformation_handler,
            // graph
            get_graph_handler,
            get_graphs_handler,
            post_graph_handler,
            // infra
            get_infras_handler,
            // table feature view
            get_table_feature_view_handler,
            get_table_feature_views_handler,
            post_table_feature_view_handler,
            // topology feature view
            get_topology_feature_view_handler,
            get_topology_feature_views_handler,
            post_topology_feature_view_handler,
            // single graph dataset
            get_graph_dataset_handler,
            get_graph_datasets_handler,
            post_graph_dataset_handler,
            // provider
            get_entities_provider_handler,
            get_fields_via_post_entity_provider_handler,
            // gaf and configs
            get_gaf_handler,
            get_configs_handler,
            // topology
            get_topology_handler,
            get_topologies_handler,
            post_topology_handler,
        ],
    )
}
