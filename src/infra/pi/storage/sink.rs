use super::source::Collector;
use crate::SeResult;

#[async_trait::async_trait(?Send)]
pub trait Writer<T> {
    async fn write(&mut self, record: T) -> SeResult<()>;
}

pub struct WriteCollector<'a, T> {
    writer: Box<dyn Writer<T> + 'a>,
}

#[async_trait::async_trait(?Send)]
impl<'a, T> Collector<T> for WriteCollector<'a, T> {
    async fn collect(&mut self, record: T) -> SeResult<()> {
        self.writer.write(record).await
    }
}

#[async_trait::async_trait(?Send)]
pub trait Sink<T>: std::fmt::Debug {
    async fn create_writer(&self) -> SeResult<Box<dyn Writer<T> + '_>>;

    async fn create_write_collector(&self) -> SeResult<WriteCollector<T>> {
        Ok(WriteCollector {
            writer: self.create_writer().await?,
        })
    }
}
