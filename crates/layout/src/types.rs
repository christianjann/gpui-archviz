use crate::grid::Grid;
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
    fn source_port(&self) -> Option<usize>;
    fn target_port(&self) -> Option<usize>;
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
    pub id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub size: Size,
    pub position: Position,
    pub ports: Vec<Port>,                  // up to 8 ports
    pub attributes: Vec<(String, String)>, // optional key-value attributes
}

impl Node {
    /// Returns the effective bounding box including ports, as (min_x, max_x, min_y, max_y) relative to node position
    pub fn effective_bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = 0.0;
        let mut max_x = self.size.width;
        let mut min_y = 0.0;
        let mut max_y = self.size.height;

        for port in &self.ports {
            min_x = f64::min(min_x, port.position.x);
            max_x = f64::max(max_x, port.position.x + port.size.width);
            min_y = f64::min(min_y, port.position.y);
            max_y = f64::max(max_y, port.position.y + port.size.height);
        }

        (min_x, max_x, min_y, max_y)
    }

    /// Returns the effective size including ports
    pub fn effective_size(&self) -> Size {
        let (min_x, max_x, min_y, max_y) = self.effective_bounds();
        Size {
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
    pub source_port: Option<usize>,
    pub target_port: Option<usize>,
    pub path: Vec<Position>, // waypoints for routing
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub grid: Option<Grid>,
}

#[derive(Clone)]
pub struct ArchVizLayout {
    pub iterations: usize,
    pub repulsion_strength: f64,
    pub attraction_strength: f64,
    pub initial_spacing: f64,
    pub min_spacing: f64,
    pub allow_diagonals: bool,
    pub spaced_edges: bool,
}

impl Default for ArchVizLayout {
    fn default() -> Self {
        Self {
            iterations: 100,
            repulsion_strength: 1000.0,
            attraction_strength: 0.1,
            initial_spacing: 100.0,
            min_spacing: 20.0,
            allow_diagonals: true,
            spaced_edges: false,
        }
    }
}
