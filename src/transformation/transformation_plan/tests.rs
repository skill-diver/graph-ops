use crate::{
    entity, fields,
    transformation::{
        CommonTransformationArgs, GraphComputationOps, SingleGraph, TransformationContext,
        TransformationData, Variant,
    },
    DataFrameBase, Entity, FeatureValueType, Field, InfraIdentifier, SeResult,
};

struct GraphResources(Vec<(Entity, Vec<Field>)>, Vec<Entity>);

fn define_graph_resources() -> SeResult<GraphResources> {
    let reviewer = entity!(
        "test_pipeline_reviewer",
        Variant::Default(),
        "Reviewer",
        "reviewerID"
    );
    let product = entity!(
        "test_pipeline_product",
        Variant::Default(),
        "Product",
        "asin"
    );

    let product_fields = fields!(
        vec![
            ("asin", FeatureValueType::String),
            ("price", FeatureValueType::Float),
            ("rank1", FeatureValueType::Int),
            ("rank2", FeatureValueType::Int)
        ],
        &product,
        Variant::Default(),
        None,
    );

    let reviewer_fields = fields!(
        vec![("reviewerId", FeatureValueType::String)],
        &reviewer,
        Variant::Default(),
        None,
    );

    let also_view = entity!(
        "test_pipeline_alsoView",
        Variant::Default(),
        "alsoView",
        &product,
        &product
    );
    let also_buy = entity!(
        "test_pipeline_alsoBuy",
        Variant::Default(),
        "alsoBuy",
        &product,
        &product
    );
    let is_similar_to = entity!(
        "test_pipeline_isSimilarTo",
        Variant::Default(),
        "isSimilarTo",
        &product,
        &product
    );
    let rates = entity!(
        "test_pipeline_rates",
        Variant::Default(),
        "rates",
        &reviewer,
        &product
    );
    let same_rates = entity!(
        "test_pipeline_sameRates",
        Variant::Default(),
        "sameRates",
        &reviewer,
        &reviewer
    );

    Ok(GraphResources(
        vec![(reviewer, reviewer_fields), (product, product_fields)],
        vec![also_view, also_buy, is_similar_to, same_rates, rates],
    ))
}

/// Given the source graph in neo4j, the pipeline under test is specified to compute page ranks
/// in neo4j explicity and to compute triangle counts in the same infra where source graph resides.
/// The results of pagerank are to be output to redis, while those of triangle counting are to be
/// written back to neo4j.
#[test]
fn specify_executing_infra() {
    let source_infra = InfraIdentifier::Neo4j("graph_transform".to_string());
    let sink_infra = InfraIdentifier::Redis("feature_storage".to_string());

    let graph_resources = define_graph_resources().unwrap();
    let vertex_fields = graph_resources.0;
    let edges = graph_resources.1;
    let vertices = vertex_fields
        .iter()
        .map(|(v, _)| v.clone())
        .collect::<Vec<_>>();

    // define transformation
    let (tc, page_rank_id, triangle_count_id) = {
        let tc = TransformationContext::new();
        let g = SingleGraph::new(
            &tc,
            vertex_fields,
            edges.iter().map(|e| (e.clone(), Vec::new())).collect(),
            source_infra.clone(),
        );

        let reviewer = vertices.get(0).unwrap();
        let same_rates = edges.get(3).unwrap();
        let page_rank_df = g
            .page_rank(
                vec![reviewer.clone(), same_rates.clone()],
                reviewer.clone(),
                None, // Damping Factor, default: 0.85
                None, // Max iteration, default: 20
                None, // Tolerance, default : 1e-7
                Some(CommonTransformationArgs::new(Some(source_infra.clone()))), // use neo4j to run pagerank
            )
            .unwrap();
        page_rank_df.export(&sink_infra);
        let triangle_count_df = g
            .triangle_count(
                vec![reviewer.clone(), same_rates.clone()],
                reviewer.clone(),
                None, // use default infra, i.e. source graph infra
            )
            .unwrap();
        triangle_count_df.export(&source_infra);
        (
            tc,
            page_rank_df.get_data_id(),
            triangle_count_df.get_data_id(),
        )
    };

    // check transformation
    let plan = tc
        .borrow_mut()
        .get_materialization_plan(vec![page_rank_id, triangle_count_id]);
    let page_rank_op = plan.get_op(page_rank_id).unwrap();
    let triangle_counting_op = plan.get_op(triangle_count_id).unwrap();
    let page_rank_execution_infra = page_rank_op.get_common_args().infra_id();
    let triangle_counting_execution_infra = triangle_counting_op.get_common_args().infra_id();
    assert!(
        page_rank_execution_infra == Some(&source_infra),
        "{page_rank_execution_infra:?} was specified to use {source_infra:?}"
    );
    assert!(
        triangle_counting_execution_infra == Some(&source_infra),
        "{triangle_counting_execution_infra:?} should default to use {source_infra:?}"
    );
}
