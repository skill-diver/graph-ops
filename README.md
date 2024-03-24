# Ofnil Graph Feature Platform

[![slack](https://img.shields.io/badge/slack-ofnil-brightgreen?logo=slack)](https://join.slack.com/t/ofnil/shared_invite/zt-1j6d9k0bb-cwH_BfR_3CFJ68hf6BvKaw)

> This project is under active development and we may break API compatibility in the future. We welcome contributions and community feedback.

Ofnil is a graph feature platform, which creates, manages and serves graph features for graph machine learning.

## Features

### Connect your graph database to graph machine learning

Currently, using Graph Neural Networks (GNNs) on graph databases (e.g., Neo4j) requires tedious work to prepare data to be consumed by GNN frameworks (e.g., DGL, PyG). Ofnil provides a unified graph feature definition and feature engineering pipeline, which can be used to connect your graph database effortlessly to downstream graph machine learning.

By defining and deploying your graph feature definition in Ofnil, you can save tremendous efforts of writing, maintaining and debugging the tedious data mapper, connector, dataloader, sampler, data pipeline code, etc. You also need not worry about the graph feature consistency, correctness, or different graph database infrastructure behaviors. Therefore, you can now simply focus on your graph feature definition and graph machine learning models that are needed by your business/applications.

### Define graph features in an intuitive and declarative way

```rust
// The code below is for the purpose of demonstration only. You may refer to examples/quickstart/main.rs for executable code.

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

## Upcoming Issues

- [ ] Real-time graph feature support for streaming graph update (with a unified consistent graph feature definition for both batch and streaming).
- [ ] Graph feature Point-in-Time (PIT) correctness.
- [ ] Advanced feature engineering pipeline analysis, e.g., data lineage, feature monitoring, feature discovery, etc.
- [ ] Graph feature as a Service, hide difficult and tedious details of graph feature engineering and serving with just a few lines of graph feature definition.
- [ ] ...

## Getting Started

- [Contribution Guide](./CONTRIBUTING.md)
- [Quickstart](./examples/quickstart/)
- [Docs](./docs/)
- [Ofnil Rust API Docs](https://rustdoc.ofnil.io/ofnil/)
