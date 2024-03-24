#[cfg_attr(feature = "dashboard", derive(Default))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BetweennessCentralityArgs {
    /// The number of source vertices to consider for computing centrality scores. No sampling if None.
    #[cfg_attr(feature = "dashboard", serde(rename = "sampling_size,number"))]
    pub sampling_size: Option<u32>,
    /// Seed for random number generator for sampling.
    #[cfg_attr(feature = "dashboard", serde(rename = "sampling_seed,number"))]
    pub sampling_seed: Option<u32>,
}
