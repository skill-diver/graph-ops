pub enum RandomWalkPath {
    Repeat(String, u32),   // edge type, length
    MetaPath(Vec<String>), // edge types
}
