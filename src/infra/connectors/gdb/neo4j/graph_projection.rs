use super::{GraphProjectionArgs, Neo4jDatabaseProvider, Neo4jDatabaseProviderError};
use crate::{infra::pi::transformation::GAF, SeResult};
use bb8_bolt::bolt_client::Metadata;
use log::info;
use std::sync::Arc;

/// Handles neo4j graph projection
/// TODO(tatiana): manage graph projections in the database provider?
pub(crate) struct GraphProjection<'a> {
    graph_name: String,
    args: &'a GraphProjectionArgs,
    db: &'a Arc<Neo4jDatabaseProvider>,
}

impl<'a> GraphProjection<'a> {
    pub(super) fn new(
        db: &'a Arc<Neo4jDatabaseProvider>,
        args: &'a GraphProjectionArgs,
        func: GAF,
    ) -> Self {
        Self {
            args,
            db,
            graph_name: format!("{}_{}", func, chrono::Utc::now().timestamp()),
        }
    }

    pub(super) async fn drop_graph_if_exists(&self) -> SeResult<()> {
        if self
            .db
            .check_named_graph_exists(self.graph_name.as_str())
            .await?
        {
            info!("Named graph {} exists, dropping", self.graph_name);
            self.db
                .drop_named_graph(self.graph_name.as_str())
                .await
                .unwrap();
        } else {
            info!("Named graph {} does not exist, continue", self.graph_name);
        }
        Ok(())
    }

    pub(super) async fn project_graph_unchecked(&self) -> SeResult<()> {
        // create projection graph
        let named_graph_query = format!(
            "CALL gds.graph.project(
            '{}',
            [{}],
            [{}]
          ) YIELD graphName, nodeCount, relationshipCount",
            &self.graph_name,
            self.args
                .vertices
                .iter()
                .map(|(label, _)| format!("'{label}'"))
                .collect::<Vec<_>>()
                .join(","),
            if self.args.make_edges_undirected {
                self.args
                    .edges
                    .iter()
                    .map(|edge| format!("{{{}: {{ orientation: 'UNDIRECTED' }} }}", &edge.tlabel))
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                self.args
                    .edges
                    .iter()
                    .map(|edge| format!("'{}'", &edge.tlabel))
                    .collect::<Vec<_>>()
                    .join(",")
            },
        );

        let mut bolt_conn = self.db.get_bolt_connection().await?;
        bolt_conn.run(named_graph_query.clone(), None, None).await?;
        let (records, msg) = bolt_conn
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        if records.len() != 1 {
            /* TODO(tatiana): parse and handle error (normally retry 3 times if the error is occasional).
            handle the case of empty result specially */
            Err(Box::new(Neo4jDatabaseProviderError::GraphProjection {
                query: named_graph_query,
                error_msg: format!("{msg:?}"),
            }))
        } else {
            Ok(())
        }
    }

    #[inline]
    pub(super) async fn project_graph(&self) -> SeResult<&str> {
        self.drop_graph_if_exists().await?;
        self.project_graph_unchecked().await?;
        Ok(&self.graph_name)
    }
}
