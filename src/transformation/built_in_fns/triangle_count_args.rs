#[cfg_attr(feature = "dashboard", derive(Default))]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TriangleCountArgs {
    /// Vertices with a degree higher than this will not be considered by the algorithm. Their triangle count will be -1.
    #[cfg_attr(feature = "dashboard", serde(rename = "max_degree,number"))]
    pub max_degree: Option<u64>,
}
