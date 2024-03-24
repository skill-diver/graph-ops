mod schema_provider;
pub mod storage;
pub mod transformation;

use crate::transformation::TransformationArgs;
pub use schema_provider::SchemaProvider;
pub use storage::*;
pub use transformation::gdb::*;
pub use transformation::*;

/// Required trait for all sink infra connectors. Creates a sink to consume data from supported sources.
pub trait Sinkable {
    // TODO(tatiana): better strategy for selecting data transport storage type
    /// We assume the storage types in the front is preferred to the types in the back
    fn get_supported_sources(&self) -> Vec<Storage>;

    fn get_sink(&self, src_storage: &Storage, type_info: Schema) -> SinkType {
        assert!(self.supports_source(src_storage));
        match src_storage {
            Storage::OfnilRow => SinkType::Row(self.insert_rows(type_info)),
            _ => unimplemented!("Unsupported yet"),
        }
    }

    /// Implemented for supporting reading from virtual in-memory storage `Storage::OfnilRow`
    fn insert_rows(&self, _type_info: Schema) -> Box<dyn Sink<Row>> {
        unimplemented!("Not supported")
    }

    fn supports_source(&self, source: &Storage) -> bool {
        self.get_supported_sources().contains(source)
    }
}

/// Required trait for all source infra connectors. Creates a source to provide data for supported sinks.
pub trait Sourceable {
    /// We assume the storage types in the front is preferred to the types in the back
    fn get_supported_sinks(&self) -> Vec<Storage>;

    fn get_source(&self, sink_storage: &Storage) -> SourceType {
        assert!(self.supports_sink(sink_storage));
        match sink_storage {
            Storage::OfnilRow => SourceType::Row(self.produce_rows()),
            _ => panic!("Unsupported yet"),
        }
    }

    /// Implemented for supporting virtual in-memory storage `Storage::OfnilRow`
    fn produce_rows(&self) -> Box<dyn Source<Row>> {
        panic!("Not supported")
    }

    fn supports_sink(&self, sink: &Storage) -> bool {
        self.get_supported_sinks().contains(sink)
    }
}

/// Required trait for all infra connectors that serve as data storage.
/// Automatically implemented for infra connectors that implement [Sinkable] and [Sourceable].
pub trait StorageConnector: Sinkable + Sourceable + std::fmt::Debug {}
impl<T> StorageConnector for T where T: Sinkable + Sourceable + std::fmt::Debug {}

/// Required trait for all infra connectors that support feature transformation.
pub trait TransformationConnector: Sinkable + Sourceable + std::fmt::Debug {
    fn get_supported_funcs(&self) -> Vec<GraphAnalyticFunc>;

    fn get_graph_executor(
        &self,
        func: &GraphAnalyticFunc,
        args: TransformationArgs,
        source: Vec<Storage>,
        sink: Storage,
    ) -> Box<dyn GraphComputationExecutor>;

    fn supports_func(&self, func: &GraphAnalyticFunc) -> bool {
        match func {
            GAF::OneOf(funcs) => {
                for func in funcs {
                    if self.get_supported_funcs().contains(func) {
                        return true;
                    }
                }
                false
            }
            _ => self.get_supported_funcs().contains(func),
        }
    }

    // TODO(tatiana): support rules (priority for speed, resources, etc.) as parameter?
    /// Select an output sink storage type for transformation from available sinks
    fn select_sink(&self, available: Vec<Storage>) -> Storage {
        debug_assert!(!available.is_empty());
        available.into_iter().next().unwrap()
    }

    // TODO(tatiana): support rules (priority for speed, resources, etc.) as parameter?
    /// Select an input source storage type for transformation from available sources
    fn select_source(&self, available: Vec<Storage>) -> Storage {
        debug_assert!(!available.is_empty());
        available.into_iter().next().unwrap()
    }
}
