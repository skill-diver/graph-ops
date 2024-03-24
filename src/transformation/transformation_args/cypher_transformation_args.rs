use crate::infra::pi::Schema;

#[derive(Debug, Clone)]
pub struct CypherTransformationArgs {
    pub query: String,
    pub output_schema: Schema,
}

impl CypherTransformationArgs {
    pub fn new(query: String, output_schema: Schema) -> Self {
        Self {
            query,
            output_schema,
        }
    }
}
