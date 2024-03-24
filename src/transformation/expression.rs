pub trait VertexFilter {
    fn eval(&self) -> bool;
}
