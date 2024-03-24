use async_trait::async_trait;
use bb8_bolt::{
    bb8::PooledConnection,
    bolt_client::*,
    bolt_proto,
    bolt_proto::{message::*, Message},
    Manager,
};

use log::warn;
use std::boxed::Box;
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
struct EachResult {
    data: Vec<Vec<String>>,
    has_more: bool,
}

#[derive(thiserror::Error, Debug)]
enum ResultError {
    #[error("Return empty")]
    ReturnEmptyErr,
    #[error("Null value in return")]
    NullinReturnErr,
    #[error("Std Error")]
    StdError,
}

#[async_trait]
trait ResultHandler {
    async fn parse_result(&self) -> Vec<String>;
    async fn gdb_pull(&mut self) -> Result<(Vec<Record>, Message), Box<dyn Error>>;
    async fn process(&mut self) -> Result<(), Box<dyn Error>>;
    async fn next(&mut self) -> Result<EachResult, ResultError> {
        let (records, response) = self.gdb_pull().await.map_err(|_| ResultError::StdError)?;
        let res = Success::try_from(response);
        let mut has_more = true;
        if let Ok(success) = res {
            match success.metadata().get("has_more") {
                Some(bolt_proto::Value::Boolean(v)) => {
                    has_more = *v;
                }
                _ => {
                    has_more = false;
                }
            }
        }
        let mut binding = Vec::new();
        let fields = match records.len() {
            0 => {
                warn!("Query returns empty!");
                return Err(ResultError::ReturnEmptyErr);
            }
            _ => records,
        };
        for item in fields {
            let mut item_string = Vec::new();
            for it in item.fields() {
                let item_ = match Some(it.to_owned()) {
                    Some(bolt_proto::Value::String(p)) => p,
                    _ => {
                        warn!("There is Null in the query return!");
                        return Err(ResultError::NullinReturnErr);
                    }
                };
                item_string.push(item_);
            }
            binding.push(item_string);
        }
        let each_result = EachResult {
            data: binding,
            has_more,
        };
        Ok(each_result)
    }
}

// TODO(tatiana): refactor as a sink writer of the neo4j connector
struct CSVWriter<'a> {
    bolt_conn: PooledConnection<'a, Manager>,
    path: String,
    pulling_size: i32, //size of pulling buffer
    msg: Message,
    status: bool,
}

#[async_trait]
impl<'a> ResultHandler for CSVWriter<'a> {
    async fn parse_result(&self) -> Vec<String> {
        let binding = Success::try_from(self.msg.clone()).unwrap();
        let fields = binding.metadata().get("fields");
        let mut binding = Vec::new();
        match fields {
            Some(bolt_proto::Value::List(t)) => {
                for item in t {
                    let item_ = match Some(item) {
                        Some(bolt_proto::Value::String(p)) => p,
                        _ => panic!("result handler parser error: key is not string",),
                    };
                    binding.push(item_.to_owned());
                }
            }
            _ => panic!("result handler parser error: do not have 'fields' key"),
        };
        binding
    }

    async fn gdb_pull(&mut self) -> Result<(Vec<Record>, Message), Box<dyn Error>> {
        let (records, response) = self
            .bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", self.pulling_size)])))
            .await?;
        Ok((records, response))
    }

    async fn process(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.status {
            return Ok(());
        }
        let mut has_more = true;
        let mut csv = csv::Writer::from_path(self.path.clone())?;
        let head = self.parse_result().await;
        csv.write_record(head)?;
        while has_more {
            let each_result = self.next().await;
            match each_result {
                Ok(each_result) => {
                    has_more = each_result.has_more;
                    for each_line in each_result.data {
                        let mut v_list = Vec::new();
                        for v in each_line {
                            v_list.push(v.as_str().to_owned());
                        }

                        csv.write_record(v_list)?;
                    }
                }
                Err(e) => {
                    warn!("{:#?}", e);
                    has_more = false;
                }
            };
        }
        csv.flush()?;
        Ok(())
    }
}

impl<'a> CSVWriter<'a> {
    #[allow(dead_code)] // TODO(tatiana): to be used elsewhere
    async fn new(
        mut bolt_conn: PooledConnection<'a, Manager>,
        path: String,
        query: String,
        pulling_size: i32,
    ) -> Result<CSVWriter<'a>, Box<dyn Error>> {
        // check if the path exists
        // if not, save to the current folder.
        let mut target_path = path;
        if !Path::new(&target_path).is_dir() {
            // TODO(kaili): file name
            target_path = "./result.csv".to_string();
            warn!("file path is not a path to a file, save to ./tmp.csv");
        } else {
            target_path = format!("{target_path}/result.csv");
        }

        // check whether the database is empty or the cypher query is problematic.
        let mut explain_query = query.clone();
        if !query.to_lowercase().starts_with("explain") {
            explain_query = format!("EXPLAIN {query}");
        }
        bolt_conn.run(explain_query, None, None).await?;
        let (_, explain_msg) = bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await?;

        let msg = bolt_conn.run(query, None, None).await?;

        match Success::try_from(explain_msg) {
            Ok(_) => Ok(CSVWriter {
                bolt_conn,
                path: target_path,
                pulling_size,
                msg,
                status: true,
            }),
            Err(_) => Err("The query is not valid".into()),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_result_handler() -> std::result::Result<(), Box<dyn Error>> {
    use crate::infra::connectors::Neo4jConnector;
    // setup BoltConnection
    let uri = "localhost:7687";
    let user = "neo4j";
    let password = "ofnil";

    let neo4j_provider = Neo4jConnector::new(uri, user, password, None, None)
        .await?
        .get_database();
    let bolt_conn = neo4j_provider.get_bolt_connection().await?;

    let mut r = CSVWriter::new(
        bolt_conn,
            "./target".to_string(),
            String::from("MATCH (n:Reviewer)<-[:isWrittenBy]-(:Review)-[:rates]->(:Product)<-[:rates]-(:Review)-[:isWrittenBy]->(m:Reviewer) RETURN n.reviewerID,m.reviewerID LIMIT 10;"),
            3
        ).await?;
    r.process().await?;
    Ok(())
}
