#[derive(Clone)]
pub struct GraphEdge {
    pub source: usize,
    pub target: usize,
}

impl GraphEdge {
    pub fn new(source: usize, target: usize) -> Self {
        Self { source, target }
    }
}
