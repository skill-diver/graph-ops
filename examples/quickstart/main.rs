use ofnil::*;

use log::{info, warn};
use std::{error::Error, path::Path};

async fn register_source_resources(fs: &FeatureStore) -> Result<Graph, Box<dyn Error>> {
    let graph = fs
        .infra_manager()
        .get_schema_provider(&InfraIdentifier::Neo4j("neo4j_1".to_string()))
        .register_graph(fs.registry())
        .await?;

    Ok(graph)
}

async fn graph_feature_engineering(
    fs: &FeatureStore,
    graph: &Graph,
) -> Result<Vec<Field>, Box<dyn Error>> {
    let tc = TransformationContext::new();
    // calling build_transformation is optional, which gives a named transformation. otherwise an anonymous transformation is created
    // tc.as_ref().borrow_mut().build_transformation("demo_topo_fields_trans", None);
    let transform_graph = graph.transform(&tc, fs.registry()).await?;

    info!("transform graph {:?}", transform_graph);

    let redis_infra_id = InfraIdentifier::Redis("redis".to_string());

    // built-in transformation
    let average_price = transform_graph
        .aggregate_neighbors(
            Some(
                fs.registry()
                    .get_entity(&"default/Entity/rates/Reviewer/Product".to_string())
                    .await?,
            ),
            fs.registry()
                .get_entity(&"default/Entity/Reviewer".to_string())
                .await?,
            vec!["price".to_owned()],
            "mean",
            None,
        )?
        .export(&redis_infra_id);

    let user_pagerank = transform_graph
        .page_rank(
            fs.registry()
                .get_entities(vec![
                    &"default/Entity/Reviewer".to_string(),
                    &"default/Entity/sameRates/Reviewer/Reviewer".to_string(),
                ])
                .await?,
            fs.registry()
                .get_entity(&"default/Entity/Reviewer".to_string())
                .await?,
            None, // Damping Factor, default: 0.85
            None, // Max iteration, default: 20
            None, // Tolerance, default : 1e-7
            None, // use default infra of source graph
        )?
        .export(&redis_infra_id);

    let betweenness_centrality = transform_graph
        .betweenness_centrality(
            fs.registry()
                .get_entities(vec![
                    &"default/Entity/Reviewer".to_string(),
                    &"default/Entity/sameRates/Reviewer/Reviewer".to_string(),
                ])
                .await?,
            fs.registry()
                .get_entity(&"default/Entity/Reviewer".to_string())
                .await?,
            Some(10), // Sampling size
            Some(0),  // Sampling seed
            None,
        )?
        .export(&redis_infra_id);

    let triangle_count = transform_graph
        .triangle_count(
            fs.registry()
                .get_entities(vec![
                    &"default/Entity/Reviewer".to_string(),
                    &"default/Entity/sameRates/Reviewer/Reviewer".to_string(),
                ])
                .await?,
            fs.registry()
                .get_entity(&"default/Entity/Reviewer".to_string())
                .await?,
            None,
        )?
        .export(&redis_infra_id);

    let item_prop = transform_graph
        .aggregate_neighbors(
            Some(
                fs.registry()
                    .get_entity(&"default/Entity/rates/Reviewer/Product".to_string())
                    .await?,
            ),
            fs.registry()
                .get_entity(&"default/Entity/Reviewer".to_string())
                .await?,
            vec!["rank1".to_owned(), "rank2".to_owned()],
            "mean",
            None,
        )?
        .export(&redis_infra_id);

    let new_fields: Vec<Field> = Vec::new()
        .into_iter()
        .chain(average_price)
        .chain(item_prop)
        .chain(user_pagerank)
        .chain(betweenness_centrality)
        .chain(triangle_count)
        .collect();

    finalize_transformation(
        fs,
        &tc,
        Vec::<&Entity>::new(),
        new_fields.iter().collect(),
        Vec::<&Topology>::new(),
    )
    .await?;

    Ok(new_fields)
}

// serve all features created
async fn graph_feature_serving(
    fs: &FeatureStore,
    graph: &Graph,
    fields: Vec<Field>,
) -> Result<GraphDataset, Box<dyn Error>> {
    // create graph dataset for training
    let user = fields.first().unwrap().entity_id.as_ref().unwrap();

    let topos = graph.project_topology(vec!["sameRates", "rates"]);
    let dataset = GraphDataset::new(
        "fraud_detection_train_dataset",
        vec![TableFeatureView::default(
            "fraud_detection_train_user_features",
            user.clone(),
            &fields,
        )],
        vec![TopologyFeatureView::default(
            "fraud_detection_train_topo",
            &topos,
        )],
        serving::GraphDatasetRenderingOptions::default(), // sample_k_hop_neighbors(2, vec![5, 3], None, true),
    );
    info!("graph dataset {dataset:?}");
    fs.registry()
        .register_resources(&topos.iter().collect())
        .await?;
    fs.registry().register_resource(&dataset).await?;
    Ok(dataset)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Fraud Detection Demo");

    let ofnil_home = Path::new(file!()).parent().unwrap();
    let fs = FeatureStore::init(ofnil_home.to_str()).await?;

    let graph = register_source_resources(&fs).await?;

    if graph.entity_ids.is_empty() {
        warn!("graph does not contain any entity: {:?}", graph);
        return Ok(());
    }

    let fields = graph_feature_engineering(&fs, &graph).await?;

    let dataset = graph_feature_serving(&fs, &graph, fields).await?;

    // deploy graph dataset
    // TODO(tatiana): use AirFlow for (batch) dag deployment?
    use ofnil::feature::ResourceOp;
    fs.deploy(dataset.resource_id()).await?;

    Ok(())
}
