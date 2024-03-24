# Graph-ops 

Graph-ops is a graph characterization platform that simplifies the process of connecting graph databases to graph machine learning.

## Features

### Seamless connection between graph databases and graph machine learning

Graph-ops provides a simplified set of graph feature definition and feature engineering processes, making it easy to prepare data from graph databases (e.g. Neo4j) to graph machine learning frameworks (e.g. DGL, PyG). By defining graph features, users can avoid writing complex data processing code and focus on business requirements for graph features and machine learning models.

### Define graph features in the easist way

```rust

let average_price = graph
    .aggregate_neighbors(Some(product_entity), reviewer_entity, vec!["price"], "mean")?
    .export(&redis_infra_id);

let user_pagerank = graph
    .page_rank(
        vec![reviewer_entity, sameRates_entity],
        reviewer_entity,
        None, // Damping factor, default: 0.85
        None, // Max iteration, default: 20
        None, // Tolerance, default: 1e-7
    )?
    .export(&redis_infra_id);

let triangle_count = graph
    .triangle_count(vec![reviewer_entity, sameRates_entity], reviewer_entity)?
    .export(&redis_infra_id);

let item_prop = transform_graph
    .aggregate_neighbors(
        Some(product_entity),
        reviewer_entity,
        vec!["rank1", "rank2"],
        "mean",
    )?
    .export(&redis_infra_id);
```

## Support

Graph-ops supports real-time graph feature updates, ensures point-in-time correctness (PIT) of graph features, and enhances advanced analytics capabilities for feature engineering, such as data pedigree, feature monitoring and discovery. We are also working on simplifying graph feature engineering and services into a few lines of code of graph features-as-a-service to hide the complex and tedious details behind the scenes.

## Getting Started

- [Quickstart](./examples/quickstart/)
- [Docs](./docs/)
