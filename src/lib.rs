mod config;
mod feature_registry;
mod feature_store;
mod python;

pub mod feature;
pub mod infra;
#[cfg(feature = "serving")]
pub mod serving;
pub mod transformation;

// re-export commonly used items to alleviate user import burden only
pub use feature::{
    Entity, FeatureValueType, Field, Graph, GraphDataset, TableFeatureView, Topology,
    TopologyFeatureView, TopologyType, Transformation, Variant,
};
pub use feature_registry::{FeatureRegistry, RegistryError};
pub use feature_store::FeatureStore;
pub use infra::{pi::SchemaProvider, Infra, InfraIdentifier, InfraManager};
pub use transformation::{
    finalize_transformation, DataFrameBase, GraphBase, GraphComputationOps, TransformationContext,
};

/// Result with std error
pub type SeResult<T> = Result<T, Box<dyn std::error::Error>>;
