use super::Schema;
use crate::SeResult;

#[async_trait::async_trait(?Send)]
pub trait Collector<T> {
    async fn collect(&mut self, record: T) -> SeResult<()>;
}

#[async_trait::async_trait(?Send)]
pub trait Reader<T> {
    async fn next(&mut self, output: &mut dyn Collector<T>) -> SeResult<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait Source<T>: Sync + std::fmt::Debug + std::marker::Send {
    /// Each reader reads the results once
    async fn create_reader(&self) -> SeResult<Box<dyn Reader<T> + '_>>;

    fn get_schema(&self) -> &Schema;
}
