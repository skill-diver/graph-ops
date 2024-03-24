use super::neo4j_database_provider::Neo4jDatabaseProvider;
use crate::{
    infra::pi::storage::{EdgeSchema, Row, RowCell, Sink, Writer},
    SeResult,
};
use bb8_bolt::{
    bb8::PooledConnection,
    bolt_client::{Metadata, Params},
    bolt_proto::message::Success,
    Manager,
};
use std::{collections::HashMap, sync::Arc};

// TODO(tatiana): test
#[derive(Debug)]
pub(super) struct Neo4jEdgeSink {
    db: Arc<Neo4jDatabaseProvider>,
    edge_schema: EdgeSchema,
}

impl Neo4jEdgeSink {
    pub(super) fn new(db: Arc<Neo4jDatabaseProvider>, edge_schema: EdgeSchema) -> Self {
        Self { db, edge_schema }
    }
}

#[async_trait::async_trait(?Send)]
impl Sink<Row> for Neo4jEdgeSink {
    async fn create_writer(&self) -> SeResult<Box<dyn Writer<Row> + '_>> {
        Ok(Box::new(Neo4jEdgeWriter {
            bolt_conn: self.db.get_bolt_connection().await?,
            edge_schema: &self.edge_schema,
        }))
    }
}

pub(super) struct Neo4jEdgeWriter<'a> {
    bolt_conn: PooledConnection<'a, Manager>,
    edge_schema: &'a EdgeSchema,
}

#[async_trait::async_trait(?Send)]
impl<'a> Writer<Row> for Neo4jEdgeWriter<'a> {
    async fn write(&mut self, record: Row) -> SeResult<()> {
        // the row should contain src, dst and possibly edge fields
        debug_assert_eq!(
            record.len(),
            self.edge_schema.edge_info.field_names.len() + 2
        );

        // TODO(tatiana): support bolt_proto::Value::Node equivalent RowCell type for src & dst vertices
        let src_external_id = match record.get(0) {
            RowCell::String(src_id) => src_id.clone(),
            _ => panic!("The src vertex of the edge is expected"),
        };
        let dst_external_id = match record.get(1) {
            RowCell::String(dst_id) => dst_id.clone(),
            _ => panic!("The dst vertex of the edge is expected"),
        };
        let edge_properties = self
            .edge_schema
            .edge_info
            .field_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (name.clone(), record.get(idx + 2).to_string()))
            .collect::<HashMap<_, _>>();

        let msg = self
            .bolt_conn
            .run(
                format!(
                    "MERGE (src:{src_tlabel} {{ {src_primary_key}: {src_external_id}}})-[e:{edge_tlabel} $props]-{direction}(dst:{dst_tlabel} {{ {dst_primary_key}: {dst_external_id}}})
                     RETURN id(src), id(dst);",
                     src_tlabel = self.edge_schema.src_vertex_tlabel,
                     src_primary_key = self.edge_schema.src_vertex_primary_key,
                     edge_tlabel = self.edge_schema.edge_info.tlabel.as_ref().unwrap(),
                     direction= if self.edge_schema.directed {">"} else {""},
                     dst_tlabel = self.edge_schema.dst_vertex_tlabel,
                     dst_primary_key = self.edge_schema.dst_vertex_primary_key
                ),
                Some(Params::from_iter(vec![("props", edge_properties)])),
                None,
            )
            .await
            .unwrap();
        assert!(Success::try_from(msg).is_ok(), "Execute MERGE query failed");
        let (_, msg) = self
            .bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        assert!(Success::try_from(msg).is_ok(), "Execute MERGE query failed");

        Ok(())
    }
}
