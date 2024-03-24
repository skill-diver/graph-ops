use crate::transformation::{GraphSchema, InputSchema, TransformationOutputHandler};

/// Trait for cypher query support
#[async_trait::async_trait(?Send)]
pub trait CypherSupport {
    async fn parse_query(
        &self,
        query: &str,
    ) -> Result<Box<dyn QueryParser>, Box<dyn std::error::Error>>;

    async fn execute_query(
        &self,
        query: &str,
    ) -> Result<TransformationOutputHandler, Box<dyn std::error::Error>>;
}

#[derive(thiserror::Error, Debug)]
pub enum QueryParserError {
    #[error("Error in {provider}. {code}: {message}")]
    GDBError {
        provider: String,
        code: String,
        message: String,
    },
    #[error("{0}")]
    ConnectorError(Box<dyn std::error::Error>),
    #[error("{0}")]
    UnsupportedQuery(String),
    #[error("Expected at least {0}, got {1} fields.")]
    ReturnFieldsError(usize, usize),
}

pub trait QueryParser {
    /// Validates the query. A QueryParserError is thrown to indicate the problems in case of invalid query.
    /// If a feature registry is given, the query inputs are checked against registered resources. A query is invalid if it requires inputs that are not registered in the registry.
    fn validate_query(
        &self,
        input_schema: Option<&InputSchema>,
        required_fields: usize,
    ) -> Result<(), QueryParserError>;

    fn get_output_graph_schema(
        &self,
        input_schema: &InputSchema,
    ) -> Result<GraphSchema, QueryParserError>;
}

pub trait QueryResultIterator<T> {
    fn next(&self) -> Option<T>;
}

pub trait QueryExecutor<T> {
    fn execute_query(&self) -> Box<dyn QueryResultIterator<T>>;
}
