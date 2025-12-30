use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub enum LayoutError {
    InvalidNodeIndex,
    InvalidEdgeIndex,
    LayoutFailed(String),
}

/// Trait for layout-compatible data structures
pub trait LayoutNode {
    fn position(&self) -> Position;
    fn size(&self) -> Size;
    fn set_position(&mut self, pos: Position);
    fn id(&self) -> String;
    fn ports(&self) -> Vec<Port>;
}

/// Trait for layout-compatible edge structures
pub trait LayoutEdge {
    fn source(&self) -> usize;
    fn target(&self) -> usize;
    fn set_path(&mut self, path: Vec<Position>);
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub position: Position, // relative to node position
    pub size: Size,
    pub port_type: PortType,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub size: Size,
    pub position: Position,
    pub ports: Vec<Port>,                  // up to 8 ports
    pub attributes: Vec<(String, String)>, // optional key-value attributes
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub path: Vec<Position>, // waypoints for routing
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

#[derive(Clone)]
pub struct CustomLayout {
    pub iterations: usize,
    pub repulsion_strength: f64,
    pub attraction_strength: f64,
    pub min_spacing: f64,
    pub allow_diagonals: bool,
    pub spaced_edges: bool,
}

impl Default for CustomLayout {
    fn default() -> Self {
        Self {
            iterations: 100,
            repulsion_strength: 1000.0,
            attraction_strength: 0.1,
            min_spacing: 20.0,
            allow_diagonals: true,
            spaced_edges: false,
        }
    }
}
