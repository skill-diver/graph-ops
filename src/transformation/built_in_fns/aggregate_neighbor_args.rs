use crate::transformation::dataframes::AggregateFunc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AggregateNeighborArgs {
    pub func: AggregateFunc,
    #[cfg_attr(feature = "dashboard", serde(rename = "properties,string[]"))]
    pub properties: Vec<String>,
}

impl Default for AggregateNeighborArgs {
    fn default() -> Self {
        Self {
            func: AggregateFunc::Count,
            properties: Vec::new(),
        }
    }
}
