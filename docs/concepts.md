# Concepts: Feature Registry Explained

> **TODO**
>
> 1. Add examples for resource definitions
> 2. Discuss the correspondence between resources and data representations in the transformation context

Ofnil uses a registry to store all feature-related **resources** (definitions, e.g., `Entity`, `Field`, `FeatureView`, etc.).

The resources include (which implements the trait ResourceOp)

1. Entity
2. Field
3. Topology
4. Graph
5. FeatureView
6. Transformation

## Concepts

We regard graph databases as the first-class data storage for the feature platform. Namely, we store graph data in a property-graph database primarily. Therefore, we also adopt the concepts in the property graph model, i.e., vertex/edge type labels (in short tlabel to differentiate from ML labels) and properties.

### 1. ResourceOp

ResourceOp implements the resource_id() function to provide a unique key for storage and retrieval in the registry.

Each ResourceOp implementation has several variables (e.g. variant, description, owners, tags, etc.) for version control, access control, meta management, etc. For current release, we can ignore them.

### 2. Entity

An entity corresponds to a type of vertices, edges, or graphs (graph entity is to be supported in later releases). Each entity can be associated with multiple features, which are semantically correlated. Each entity has a name that uniquely identifies itself, and a primary key that shows how features extracted/computed from different places can be joined together. The primary key (plus timestamp) can also be used to extract features for specific vertices/edges.

- Users can define an entity that already exists in the graph database, or an entity that is to be created during feature engineering.
- For vertex and edge entities, they have a tlabel that is used for storage in graph databases and graph queries (in graph databases and other graph engines).
- For each edge entity, it is associated with the source and destination vertex entities.

For example, we can define a user entity for the recommendation scenario, whose type label in the graph database is User and has the property user_id as its primary key.

```rust
let user = entity!(
    "recommendation_project_user", // entity name
    Variant::Default(),            // version
    "User",                        // tlabel
    "user_id"                      // primary key
)
```

### 3. Field

In Ofnil, both the numeric vectors and the graph topology are considered as features in a general sense. The traditional notion of a feature, which can be a property or a dimension in the feature vector, is defined as a field.

Each field is associated with an entity (some global features may not relate to any entity, but in the current release it is not considered).

Fields consist of:

- `name`: The field name to identify a feature of an entity: in a graph database, the field name is the property key; in a tabular representation, the field name is the column name.
- `entity_id`: The resource id of the entity.
- `value_type`: The value type.
- `transformation_id` (optional): The resource ID of the producer transformation, if the field is computed from feature transformation.
- `sink_infra_id`: The infra where the field is materialized.
- Other common ResourceOp metadata

### 4. Topology

A Topology represents a homogeneous graph, i.e., it has a single type of edges.

Topologies consist of:

- `name`: A unique topology name across the registry. (Shall we simply make it as the relationship label?)
- Edge, source vertex, and destination vertex entity IDs.
- `transformation_id` (optional): The resource ID of the producer transformation, if the topology is computed from feature transformation.
- `topology_type` (optional): The representation type, e.g., adjacency matrix, bipartite graph chain, if the topology is not stored in the primary graph database.
- `sink_infra_id`: The infra where the topology is materialized.
- Other common ResourceOp metadata

### 5. Graph

Graph is a composite definition including entities, fields, and topologies. Graph represents a single large graph (in a graph database) and contains a collection of vertex/edge entities. The associated fields of the entities are a logical part of the Graph, too. The Graph can be used in the context of transformation through Graph.transform() to allow specification of graph operations. Currently, we use Graph to represent the source graphs in the graph database(s) only. The graph databases can implement a schema query interface to provide the functionality of auto-defining entity and field resources (to be supported).

### 6. Feature View

Feature view represents a logical group of features or topologies, i.e., Field or Topology, which could be composed of the semantics of the features. FeatureView includes `TableFeatureView` and `TopologyFeatureView`.

Both `TableFeatureView` and `TopologyFeatureView` consist of:

- `name`: The feature view name to identify the feature view.
- `online`: indicate whether the serving is online or offline. Not used for now.
- `rendering_opt`: Rendering options that specify the format of the output and the serving mode. For `TableFeatureView`, the current output formats include `NdArray`; while for `TopologyFeatureView`, the current output formats include `CompressedSparseRow`. They both support Python binding mode for now.
- `sink_infra_id`: The infra where the field or topology is materialized.

In addition, `TableFeatureView` is composed of a vector of `Field`s represented by `field_ids`.

- For now, we handle only the case where all the fields come from the same entity. Namely, the `entity_id` variable is simply a shortcut that shows the entities of all constituent fields.
- For the case where not all the fields come from the same entity, the `entity_id` (possibly needs to be modified to a vector) is used to specify the entities for joins, which is similar to the join_keys in Feast.

TopologyFeatureView is composed of a vector of topology IDs, representing the (possibly heteregeneous) graph topology to be used for feature serving.

### 7. Transformation

A `Transformation` defines a collection of closely related data transformation tasks. It can be represented as a DAG of operations, including graph and relational operations. The graph operations can be built-in graph algorithms (partly supported in the current release) or cypher queries (testing and to be supported in later releases). The relational operations are pandas/Spark DataFrame like transformations.

- We differentiate resource definition and transformation data representation, e.g. `Graph` as a `ResourceOp` versus `SingleGraph` in the transformation context. This is because transformation data can be transient intermediate results only known to data engineers and contain computation logic, while resources are opaque to data scientist users and contain more metadata information for data management.
- Users explicitly enter the transformation context by calling `.transform()` on a resource definition, typically a Graph or a `TableFeatureView`, to obtain a transformation data instance.
- Users explicitly exit the transformation context and re-enter the resource definition context by calling `.export()` on transformation data, for example `DataFrame`, to obtain resource definitions.
- The specifications of transformation are declarative, i.e., no real computation occurs until the transformation is deployed.

Each transformation is defined in `TransformationContext`, which contains all data involved in a data flow and is used to construct transformation plans for data materialization. Specifically, it stores the transformation operations, parent data, and the used infra.

## Comparison with Feast

The existing feature stores or feature platforms adopt similar feature abstractions to those in Feast. We use Feast as a representative and compare our graph-oriented feature abstraction with existing table-oriented ones.

- Graph-specific differences
  1. Field as a first-class resource for the sake of flexible graph schema (Fields are defined as part of feature views. Since Feast does not transform data, a field is essentially a schema that only contains a name and a type.)
  2. Exclusive topology feature & topology feature view
  3. Entities are more than join keys, and feature joins explicitly use relationships (edge entities)
- Declarative transformation logic as a resource

## Reference

- [Feast registry](https://docs.feast.dev/getting-started/concepts/registry)
