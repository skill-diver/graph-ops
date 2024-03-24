// TODO(han): more parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PageRankArgs {
    pub damping_factor: f32,
    pub max_iteration: u32,
    pub tolerance: f32,
}

#[cfg(feature = "dashboard")]
impl Default for PageRankArgs {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iteration: 20,
            tolerance: 0.0001,
        }
    }
}
