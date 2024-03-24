use bb8_bolt::{
    bb8::PooledConnection,
    bolt_client::Metadata,
    bolt_proto::{message, Value},
    Manager,
};
use log::info;

use crate::transformation::transformation_args::CypherTransformationArgs;

use super::*;

// TODO(tatiana): test
#[derive(Debug)]
pub struct Neo4JQueryRowSource {
    db: Arc<Neo4jDatabaseProvider>,
    pull_size: i32,
    cypher_args: CypherTransformationArgs,
}

impl Neo4JQueryRowSource {
    pub(super) fn new(
        db: Arc<Neo4jDatabaseProvider>,
        cypher_args: CypherTransformationArgs,
        pull_size: i32,
    ) -> Self {
        Neo4JQueryRowSource {
            db,
            pull_size,
            cypher_args,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Source<Row> for Neo4JQueryRowSource {
    async fn create_reader(&self) -> SeResult<Box<dyn Reader<Row> + '_>> {
        Ok(Box::new(
            Neo4JQueryRowReader::new(
                self.db.get_bolt_connection().await?,
                self.cypher_args.query.clone(),
                self.pull_size,
            )
            .await?,
        ))
    }

    fn get_schema(&self) -> &Schema {
        &self.cypher_args.output_schema
    }
}

pub struct Neo4JQueryRowReader<'a> {
    bolt_conn: PooledConnection<'a, Manager>,
    pull_size: i32,
}

impl<'a> Neo4JQueryRowReader<'a> {
    async fn new(
        mut bolt_conn: PooledConnection<'a, Manager>,
        query: String,
        pull_size: i32,
    ) -> SeResult<Neo4JQueryRowReader<'a>> {
        info!("run query {query}");
        let msg = bolt_conn.run(query, None, None).await?;
        info!("run query result message {msg:?}");
        Ok(Self {
            bolt_conn,
            pull_size,
        })
    }
}

#[async_trait::async_trait(?Send)]
impl<'a> Reader<Row> for Neo4JQueryRowReader<'a> {
    async fn next(&mut self, output: &mut dyn Collector<Row>) -> SeResult<bool> {
        let (records, msg) = self
            .bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", self.pull_size)])))
            .await?;
        let success = message::Success::try_from(msg).unwrap();
        for record in records {
            if let Value::String(id) = record.fields().get(0).unwrap() {
                let row_fields = std::iter::once(RowCell::String(id.clone()))
                    .chain(record.fields().iter().skip(1).map(|field| {
                        match field {
                            // TODO(tatiana): a systematic way to handle nulls
                            Value::Null => RowCell::Null,
                            Value::Float(val) => RowCell::Double(*val),
                            Value::Boolean(val) => RowCell::Boolean(*val),
                            Value::Integer(val) => RowCell::Int(*val),
                            Value::String(val) => RowCell::String(val.clone()),
                            _ => unimplemented!("value type not supported yet, got {field:?}"),
                        }
                    }))
                    .collect();
                output.collect(Row::new(row_fields)).await?;
            } else {
                unimplemented!("now only support query that returns an ID and float field(s)")
            }
        }
        Ok(success.metadata().contains_key("has_more")
            && success.metadata()["has_more"] == Value::Boolean(true))
    }
}
