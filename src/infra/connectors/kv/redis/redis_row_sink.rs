use crate::{infra::pi::storage::*, SeResult};

use super::get_connection_arc;
use redis::{Commands, NumericBehavior, ToRedisArgs};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct RedisRowSink {
    client: Arc<Mutex<redis::Client>>,
    type_info: Schema,
}

impl RedisRowSink {
    pub(super) fn new(client: Arc<Mutex<redis::Client>>, type_info: Schema) -> Self {
        Self { client, type_info }
    }
}

#[async_trait::async_trait(?Send)]
impl Sink<Row> for RedisRowSink {
    async fn create_writer(&self) -> SeResult<Box<dyn Writer<Row> + '_>> {
        Ok(Box::new(RedisRowWriter::new(
            get_connection_arc(&self.client)?,
            self.type_info.clone(),
        )))
    }
}

pub struct RedisRowWriter {
    redis_conn: redis::Connection,
    field_names: Vec<String>,
    tlabel: String,
}

impl RedisRowWriter {
    fn new(redis_conn: redis::Connection, type_info: Schema) -> Self {
        let tabular_schema = type_info.into_tabular();
        Self {
            redis_conn,
            tlabel: tabular_schema
                .tlabel
                .expect("now assume all tabular data are associated with a vertex/edge"),
            field_names: tabular_schema.field_names,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Writer<Row> for RedisRowWriter {
    async fn write(&mut self, record: Row) -> SeResult<()> {
        // TODO(tatiana): support configs for serialization. now generating a kv pair for each non-id field in the row, using the first field as id.
        // TODO(tatiana): support timestamp
        const FIELD_OFFSET: usize = 1;
        let id = record.get(0).string();
        debug_assert_eq!(record.len(), self.field_names.len() + 1); // first element in record is id
        for (idx, name) in self.field_names.iter().enumerate() {
            let key = format!("{}/{name}/{id}", self.tlabel);
            let value = record.get(idx + FIELD_OFFSET);
            if let RowCell::Null = value {
                // TODO(tatiana): handle null
            } else {
                self.redis_conn.set(key, record.get(idx + FIELD_OFFSET))?;
            }
        }
        Ok(())
    }
}

// we do not encode the type info in redis value but rely on the schema info in registry
impl ToRedisArgs for RowCell {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            RowCell::Null => panic!("unexpected null"),
            RowCell::String(value)
            | RowCell::Date(value)
            | RowCell::Time(value)
            | RowCell::DateTime(value) => value.write_redis_args(out),
            RowCell::Float(value) => value.write_redis_args(out),
            RowCell::Double(value) => value.write_redis_args(out),
            RowCell::Int(value) => value.write_redis_args(out),
            RowCell::Boolean(value) => value.write_redis_args(out),
            RowCell::Duration(value) => value.write_redis_args(out),
            // TODO(tatiana): array need to be interpreted as a single arg if redis SET is used. Consider a more suitable way to handle array
            RowCell::Array(vec) => out.write_arg_fmt(
                vec.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        }
    }

    fn describe_numeric_behavior(&self) -> NumericBehavior {
        match self {
            RowCell::Float(_) | RowCell::Double(_) => NumericBehavior::NumberIsFloat,
            RowCell::Int(_) | RowCell::Duration(_) => NumericBehavior::NumberIsInteger,
            _ => NumericBehavior::NonNumeric,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{infra::connectors::RedisConnector, FeatureValueType};

    #[tokio::test]
    async fn test_redis_row_sink() -> SeResult<()> {
        let redis_uri = std::env::var("REDIS_URI");
        println!("redis_uri={redis_uri:?}");

        if let Ok(uri) = redis_uri {
            let sink = RedisRowSink::new(
                RedisConnector::new(uri).get_client(),
                Schema::Tabular(TabularSchema {
                    field_names: vec![
                        "double_col",
                        "float_col",
                        "int_col",
                        "bool_col",
                        "duration_col",
                    ]
                    .into_iter()
                    .map(|name| name.to_string())
                    .collect(),
                    field_types: vec![
                        FeatureValueType::Float,
                        FeatureValueType::Float,
                        FeatureValueType::Int,
                        FeatureValueType::Boolean,
                        FeatureValueType::Duration,
                    ],
                    tlabel: Some("TestEntity".to_string()),
                }),
            );
            let mut writer = sink.create_writer().await?;
            let key = "test_redis_row_sink";
            writer
                .write(Row::new(vec![
                    RowCell::String(key.to_string()),
                    RowCell::Double(0.618),
                    RowCell::Float(std::f32::consts::PI),
                    RowCell::Int(1024),
                    RowCell::Boolean(true),
                    RowCell::Duration(6174),
                ]))
                .await?;

            let mut conn = super::get_connection_arc(&sink.client)?;
            let record: f64 = conn.get_del(format!("TestEntity/double_col/{key}"))?;
            assert_eq!(record, 0.618);
            let record: f32 = conn.get_del(format!("TestEntity/float_col/{key}"))?;
            assert_eq!(record, std::f32::consts::PI);
            let record: i32 = conn.get_del(format!("TestEntity/int_col/{key}"))?;
            assert_eq!(record, 1024);
            let record: bool = conn.get_del(format!("TestEntity/bool_col/{key}"))?;
            assert!(record);
            let record: u64 = conn.get_del(format!("TestEntity/duration_col/{key}"))?;
            assert_eq!(record, 6174);
        }
        Ok(())
    }
}
