use super::{Row, Sink};

pub enum SinkType {
    Row(Box<dyn Sink<Row>>),
}
