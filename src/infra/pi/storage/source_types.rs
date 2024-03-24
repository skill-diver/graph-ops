use super::{Row, Source};

pub enum SourceType {
    Row(Box<dyn Source<Row>>),
}
