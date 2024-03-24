mod file;
mod sink;
mod sink_types;
mod source;
mod source_types;
mod tabular;
use crate::FeatureValueType;
pub use file::*;
use serde::{Deserialize, Serialize};
pub use sink::*;
pub use sink_types::*;
pub use source::*;
pub use source_types::*;
pub use tabular::*;

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum FileSystemIdentifier {
    Local,
    HDFS,
    S3,
    GCS,
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum FileFormat {
    CSV,
    Parquet,
    CSR,
    COO,
}

/// Storage types can be common-format files on a file system, in-memory data structure, or infra-specific storage.
#[derive(Eq, Hash, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum Storage {
    // common fileGbased storage
    File {
        fs: FileSystemIdentifier,
        format: FileFormat,
    },
    // common in-memory storage
    OfnilRow,
    Arrow,
    // specific storage
    Neo4j,
    Redis,
}

#[derive(Clone, Debug, enum_methods::EnumAsGetters, enum_methods::EnumIntoGetters)]
pub enum Schema {
    Tabular(TabularSchema),
    Edge(EdgeSchema),
}

#[derive(Clone, Debug)]
pub struct TabularSchema {
    pub field_names: Vec<String>,
    pub field_types: Vec<FeatureValueType>,
    /// the type/label of the associated entity
    pub tlabel: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EdgeSchema {
    pub src_vertex_tlabel: String,
    pub dst_vertex_tlabel: String,
    pub src_vertex_primary_key: String,
    pub dst_vertex_primary_key: String,
    pub directed: bool,
    pub edge_info: TabularSchema, // edge type and fields
}
