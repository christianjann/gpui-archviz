use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

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

struct Grid {
    cell_size: f64,
    origin_x: f64,
    origin_y: f64,
    width: usize,
    height: usize,
    obstacles: Vec<Vec<bool>>,
}

impl Grid {
    fn new(nodes: &[Node], _canvas_width: f64, _canvas_height: f64, cell_size: f64) -> Self {
        let min_x = nodes
            .iter()
            .map(|n| n.position.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = nodes
            .iter()
            .map(|n| n.position.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = nodes
            .iter()
            .map(|n| n.position.x + n.size.width)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = nodes
            .iter()
            .map(|n| n.position.y + n.size.height)
            .fold(f64::NEG_INFINITY, f64::max);
        let canvas_width = (max_x - min_x + 100.0).max(400.0);
        let canvas_height = (max_y - min_y + 100.0).max(300.0);
        let width = (canvas_width / cell_size).ceil() as usize + 1;
        let height = (canvas_height / cell_size).ceil() as usize + 1;
        let mut obstacles = vec![vec![false; width]; height];

        for node in nodes {
            let x1 = ((node.position.x - 7.0 - min_x) / cell_size).floor() as usize;
            let y1 = ((node.position.y - 7.0 - min_y) / cell_size).floor() as usize;
            let x2 =
                ((node.position.x + node.size.width + 7.0 - min_x) / cell_size).ceil() as usize;
            let y2 =
                ((node.position.y + node.size.height + 7.0 - min_y) / cell_size).ceil() as usize;

            for y in y1..y2.min(height) {
                for x in x1..x2.min(width) {
                    obstacles[y][x] = true;
                }
            }
        }

        // Block port areas
        for node in nodes {
            for port in &node.ports {
                let port_x1 =
                    (((node.position.x + port.position.x - min_x) / cell_size).floor() as i32 - 2)
                        .max(0) as usize;
                let port_y1 =
                    (((node.position.y + port.position.y - min_y) / cell_size).floor() as i32 - 2)
                        .max(0) as usize;
                let port_x2 = (((node.position.x + port.position.x + port.size.width - min_x)
                    / cell_size)
                    .ceil() as i32
                    + 2)
                .min((width - 1) as i32) as usize
                    + 1;
                let port_y2 = (((node.position.y + port.position.y + port.size.height - min_y)
                    / cell_size)
                    .ceil() as i32
                    + 2)
                .min((height - 1) as i32) as usize
                    + 1;

                for y in port_y1..port_y2.min(height) {
                    for x in port_x1..port_x2.min(width) {
                        obstacles[y][x] = true;
                    }
                }
            }
        }

        Grid {
            cell_size,
            origin_x: min_x,
            origin_y: min_y,
            width,
            height,
            obstacles,
        }
    }

    fn pos_to_grid(&self, pos: &Position) -> (usize, usize) {
        let x = (((pos.x - self.origin_x) / self.cell_size).round() as usize).min(self.width - 1);
        let y = (((pos.y - self.origin_y) / self.cell_size).round() as usize).min(self.height - 1);
        (x, y)
    }

    fn grid_to_pos(&self, x: usize, y: usize) -> Position {
        Position {
            x: self.origin_x + x as f64 * self.cell_size,
            y: self.origin_y + y as f64 * self.cell_size,
        }
    }

    fn find_path(&self, start: Position, end: Position) -> Vec<Position> {
        let start_grid = self.pos_to_grid(&start);
        let end_grid = self.pos_to_grid(&end);

        if start_grid == end_grid {
            return vec![start, end];
        }

        // BFS for orthogonal path
        let mut queue = VecDeque::new();
        queue.push_back(start_grid);

        let mut came_from = HashMap::new();
        came_from.insert(start_grid, None);

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == end_grid {
                found = true;
                break;
            }

            for neighbor in self.neighbors(current) {
                if !came_from.contains_key(&neighbor) {
                    queue.push_back(neighbor);
                    came_from.insert(neighbor, Some(current));
                }
            }
        }

        if !found {
            return vec![start, end];
        }

        // Reconstruct path
        let mut path = vec![end_grid];
        let mut current = end_grid;
        while let Some(prev) = came_from[&current] {
            path.push(prev);
            current = prev;
        }
        path.reverse();

        path.into_iter()
            .map(|(x, y)| self.grid_to_pos(x, y))
            .collect()
    }

    fn neighbors(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = pos;
        let mut neighbors = vec![];
        if x > 0 && !self.obstacles[y][x - 1] {
            neighbors.push((x - 1, y));
        }
        if x < self.width - 1 && !self.obstacles[y][x + 1] {
            neighbors.push((x + 1, y));
        }
        if y > 0 && !self.obstacles[y - 1][x] {
            neighbors.push((x, y - 1));
        }
        if y < self.height - 1 && !self.obstacles[y + 1][x] {
            neighbors.push((x, y + 1));
        }
        neighbors
    }
}

fn nearest_grid(pos: &Position, origin_x: f64, origin_y: f64, cell_size: f64) -> Position {
    let gx = ((pos.x - origin_x) / cell_size).round();
    let gy = ((pos.y - origin_y) / cell_size).round();
    Position {
        x: origin_x + gx * cell_size,
        y: origin_y + gy * cell_size,
    }
}

impl CustomLayout {
    pub fn layout(&self, nodes: Vec<Node>, edges: Vec<(usize, usize)>) -> LayoutResult {
        let mut nodes = nodes;
        let edges = edges;

        // Phase 1: Initial placement
        self.initial_placement(&mut nodes, &edges);

        // Phase 2: Force-directed refinement
        self.force_directed(&mut nodes, &edges);

        // Phase 3: Edge routing
        let mut routed_edges = self.route_edges(&nodes, &edges);

        // Phase 4: Calculate canvas and center layout
        let (canvas_width, canvas_height) = self.calculate_canvas_size(&nodes, &routed_edges);
        self.center_layout(&mut nodes, &mut routed_edges, canvas_width, canvas_height);

        LayoutResult {
            nodes,
            edges: routed_edges,
            canvas_width,
            canvas_height,
        }
    }

    fn initial_placement(&self, nodes: &mut [Node], edges: &[(usize, usize)]) {
        // Simple clustering based on connectivity
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; nodes.len()];

        for i in 0..nodes.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = vec![i];
            assigned[i] = true;

            // Find connected nodes
            for &(a, b) in edges {
                if a == i && !assigned[b] {
                    cluster.push(b);
                    assigned[b] = true;
                } else if b == i && !assigned[a] {
                    cluster.push(a);
                    assigned[a] = true;
                }
            }
            clusters.push(cluster);
        }

        // Place clusters
        let mut x_offset = 0.0;
        for cluster in clusters {
            let mut y_offset = 0.0;
            for &node_idx in &cluster {
                nodes[node_idx].position = Position {
                    x: x_offset,
                    y: y_offset,
                };
                y_offset += nodes[node_idx].size.height + self.min_spacing;
            }
            x_offset += cluster
                .iter()
                .map(|&i| nodes[i].size.width)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(100.0)
                + self.min_spacing;
        }
    }

    fn force_directed(&self, nodes: &mut [Node], edges: &[(usize, usize)]) {
        for _ in 0..self.iterations {
            let mut forces: Vec<Position> = vec![Position { x: 0.0, y: 0.0 }; nodes.len()];

            // Calculate repulsion
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let dx = (nodes[j].position.x + nodes[j].size.width / 2.0)
                        - (nodes[i].position.x + nodes[i].size.width / 2.0);
                    let dy = (nodes[j].position.y + nodes[j].size.height / 2.0)
                        - (nodes[i].position.y + nodes[i].size.height / 2.0);
                    let min_dist =
                        (nodes[i].size.width / 2.0 + nodes[j].size.width / 2.0 + self.min_spacing)
                            .max(
                                nodes[i].size.height / 2.0
                                    + nodes[j].size.height / 2.0
                                    + self.min_spacing,
                            );
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < min_dist {
                        let force = self.repulsion_strength * (min_dist - dist) / min_dist;
                        let fx = force * dx / dist.max(1.0);
                        let fy = force * dy / dist.max(1.0);

                        forces[i].x -= fx;
                        forces[i].y -= fy;
                        forces[j].x += fx;
                        forces[j].y += fy;
                    }
                }
            }

            // Calculate attraction
            for &(a, b) in edges {
                let dx = nodes[b].position.x - nodes[a].position.x;
                let dy = nodes[b].position.y - nodes[a].position.y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let force = self.attraction_strength * dist;

                let fx = force * dx / dist;
                let fy = force * dy / dist;

                forces[a].x += fx;
                forces[a].y += fy;
                forces[b].x -= fx;
                forces[b].y -= fy;
            }

            // Apply forces
            for i in 0..nodes.len() {
                nodes[i].position.x += forces[i].x * 0.1;
                nodes[i].position.y += forces[i].y * 0.1;
            }
        }
    }

    fn route_edges(&self, nodes: &[Node], edges: &[(usize, usize)]) -> Vec<Edge> {
        // Estimate canvas size
        let min_x = nodes
            .iter()
            .map(|n| n.position.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = nodes
            .iter()
            .map(|n| n.position.x + n.size.width)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = nodes
            .iter()
            .map(|n| n.position.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = nodes
            .iter()
            .map(|n| n.position.y + n.size.height)
            .fold(f64::NEG_INFINITY, f64::max);
        let _canvas_width = (max_x - min_x + 100.0).max(400.0);
        let _canvas_height = (max_y - min_y + 100.0).max(300.0);

        let cell_size = 5.0;
        let mut grid = Grid::new(nodes, _canvas_width, _canvas_height, cell_size);

        let mut routed_edges = vec![];
        for &(source, target) in edges {
            let source_node = &nodes[source];
            let target_node = &nodes[target];

            let source_pos = self.select_port(source_node, target_node);
            let target_pos = self.select_port(target_node, source_node);

            let source_grid_pos =
                nearest_grid(&source_pos, grid.origin_x, grid.origin_y, cell_size);
            let target_grid_pos =
                nearest_grid(&target_pos, grid.origin_x, grid.origin_y, cell_size);

            let source_grid = grid.pos_to_grid(&source_grid_pos);
            let target_grid = grid.pos_to_grid(&target_grid_pos);

            grid.obstacles[source_grid.1][source_grid.0] = false;
            grid.obstacles[target_grid.1][target_grid.0] = false;

            // Calculate direction for source based on port side
            let source_side = if (source_pos.x - source_node.position.x).abs() < 10.0 {
                "left"
            } else if (source_pos.x - (source_node.position.x + source_node.size.width)).abs() < 10.0 {
                "right"
            } else if (source_pos.y - source_node.position.y).abs() < 10.0 {
                "top"
            } else {
                "bottom"
            };

            let source_ext_x = source_grid_pos.x
                + match source_side {
                    "right" => 25.0,
                    "left" => -25.0,
                    _ => 0.0,
                };
            let source_ext_y = source_grid_pos.y
                + match source_side {
                    "bottom" => 25.0,
                    "top" => -25.0,
                    "right" => 0.0,
                    _ => 0.0,
                };
            let source_ext_end = Position {
                x: source_ext_x,
                y: source_ext_y,
            };

            let source_ext_end = nearest_grid(&source_ext_end, grid.origin_x, grid.origin_y, cell_size);
            
            // Calculate direction for target based on port side
            let target_side = if (target_pos.x - target_node.position.x).abs() < 10.0 {
                "left"
            } else if (target_pos.x - (target_node.position.x + target_node.size.width)).abs() < 10.0 {
                "right"
            } else if (target_pos.y - target_node.position.y).abs() < 10.0 {
                "top"
            } else {
                "bottom"
            };

            let target_ext_x = target_grid_pos.x
                + match target_side {
                    "right" => 25.0,
                    "left" => -25.0,
                    _ => 0.0,
                };
            let target_ext_y = target_grid_pos.y
                + match target_side {
                    "bottom" => 25.0,
                    "top" => -25.0,
                    "right" => 0.0,
                    _ => 0.0,
                };
            let target_ext_end = Position {
                x: target_ext_x,
                y: target_ext_y,
            };

            let target_ext_end = nearest_grid(&target_ext_end, grid.origin_x, grid.origin_y, cell_size);

            let source_ext_grid = grid.pos_to_grid(&source_ext_end);
            grid.obstacles[source_ext_grid.1][source_ext_grid.0] = false;

            let target_ext_grid = grid.pos_to_grid(&target_ext_end);
            grid.obstacles[target_ext_grid.1][target_ext_grid.0] = false;

            let path = grid.find_path(source_ext_end, target_ext_end);
            let mut full_path = vec![source_grid_pos];
            full_path.extend(path);
            full_path.push(target_grid_pos);

            // Debug: check for diagonal segments
            for i in 1..full_path.len() {
                let p1 = &full_path[i-1];
                let p2 = &full_path[i];
                if p1.x != p2.x && p1.y != p2.y {
                    println!("WARNING: Diagonal segment calculated: ({}, {}) to ({}, {})", p1.x, p1.y, p2.x, p2.y);
                }
            }

            routed_edges.push(Edge {
                source,
                target,
                path: full_path,
            });
        }

        routed_edges
    }

    fn select_port(&self, from_node: &Node, to_node: &Node) -> Position {
        if from_node.ports.is_empty() {
            // Default to center, snapped to grid
            let center_x = from_node.position.x + from_node.size.width / 2.0;
            let center_y = from_node.position.y + from_node.size.height / 2.0;
            let snapped_x = ((center_x / 10.0).round() * 10.0)
                .max(from_node.position.x)
                .min(from_node.position.x + from_node.size.width);
            let snapped_y = ((center_y / 10.0).round() * 10.0)
                .max(from_node.position.y)
                .min(from_node.position.y + from_node.size.height);
            Position {
                x: snapped_x,
                y: snapped_y,
            }
        } else {
            // Calculate direction vector from from_node center to to_node center
            let from_center_x = from_node.position.x + from_node.size.width / 2.0;
            let from_center_y = from_node.position.y + from_node.size.height / 2.0;
            let to_center_x = to_node.position.x + to_node.size.width / 2.0;
            let to_center_y = to_node.position.y + to_node.size.height / 2.0;

            let dx = to_center_x - from_center_x;
            let dy = to_center_y - from_center_y;

            // Find the port whose position best matches the direction
            let mut best_port = None;
            let mut best_score = f64::INFINITY;

            for port in &from_node.ports {
                let port_world_x = from_node.position.x + port.position.x + port.size.width / 2.0;
                let port_world_y = from_node.position.y + port.position.y + port.size.height / 2.0;

                let port_dx = port_world_x - from_center_x;
                let port_dy = port_world_y - from_center_y;

                // Score based on how well the port direction matches the target direction
                // Lower score is better
                let dot_product = port_dx * dx + port_dy * dy;
                let port_magnitude = (port_dx * port_dx + port_dy * port_dy).sqrt();
                let target_magnitude = (dx * dx + dy * dy).sqrt();

                if port_magnitude > 0.0 && target_magnitude > 0.0 {
                    let cos_angle = dot_product / (port_magnitude * target_magnitude);
                    let score = 1.0 - cos_angle; // 0 when perfectly aligned, 2 when opposite
                    if score < best_score {
                        best_score = score;
                        best_port = Some(port);
                    }
                }
            }

            if let Some(port) = best_port {
                Position {
                    x: from_node.position.x + port.position.x + port.size.width / 2.0,
                    y: from_node.position.y + port.position.y + port.size.height / 2.0,
                }
            } else {
                // Fallback to center if no suitable port found
                let center_x = from_node.position.x + from_node.size.width / 2.0;
                let center_y = from_node.position.y + from_node.size.height / 2.0;
                let snapped_x = ((center_x / 10.0).round() * 10.0)
                    .max(from_node.position.x)
                    .min(from_node.position.x + from_node.size.width);
                let snapped_y = ((center_y / 10.0).round() * 10.0)
                    .max(from_node.position.y)
                    .min(from_node.position.y + from_node.size.height);
                Position {
                    x: snapped_x,
                    y: snapped_y,
                }
            }
        }
    }

    fn calculate_canvas_size(&self, nodes: &[Node], edges: &[Edge]) -> (f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for node in nodes {
            min_x = min_x.min(node.position.x);
            max_x = max_x.max(node.position.x + node.size.width);
            min_y = min_y.min(node.position.y);
            max_y = max_y.max(node.position.y + node.size.height);
        }

        for edge in edges {
            for point in &edge.path {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);
            }
        }

        let width = (max_x - min_x).max(400.0) + 2.0 * self.min_spacing;
        let height = (max_y - min_y).max(300.0) + 2.0 * self.min_spacing;

        (width, height)
    }

    fn center_layout(
        &self,
        nodes: &mut [Node],
        edges: &mut Vec<Edge>,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for node in nodes.iter() {
            min_x = min_x.min(node.position.x);
            max_x = max_x.max(node.position.x + node.size.width);
            min_y = min_y.min(node.position.y);
            max_y = max_y.max(node.position.y + node.size.height);
        }

        for edge in edges.iter() {
            for point in &edge.path {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);
            }
        }

        let layout_width = max_x - min_x;
        let layout_height = max_y - min_y;
        let offset_x = (canvas_width - layout_width) / 2.0 - min_x;
        let offset_y = (canvas_height - layout_height) / 2.0 - min_y;

        for node in nodes.iter_mut() {
            node.position.x += offset_x;
            node.position.y += offset_y;
        }

        for edge in edges.iter_mut() {
            for point in &mut edge.path {
                point.x += offset_x;
                point.y += offset_y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_nodes() -> Vec<Node> {
        vec![
            Node {
                id: "A".to_string(),
                size: Size {
                    width: 100.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: 0.0, y: 25.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                    }, // left
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                    }, // right
                    Port {
                        position: Position { x: 50.0, y: 0.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                    }, // top
                    Port {
                        position: Position { x: 50.0, y: 50.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                    }, // bottom
                ],
                attributes: vec![],
            },
            Node {
                id: "B".to_string(),
                size: Size {
                    width: 80.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: 0.0, y: 30.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 80.0, y: 30.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 40.0, y: 0.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 40.0, y: 60.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ]
    }

    fn create_test_edge_indices() -> Vec<(usize, usize)> {
        vec![(0, 1)]
    }

    #[test]
    fn test_initial_placement() {
        let mut nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = CustomLayout::default();

        layout.initial_placement(&mut nodes, &edges);

        assert!(nodes[0].position.x >= 0.0);
        assert!(nodes[1].position.x >= 0.0);
    }

    #[test]
    fn test_force_directed() {
        let mut nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = CustomLayout {
            iterations: 200,
            repulsion_strength: 10000.0,
            ..Default::default()
        };

        layout.force_directed(&mut nodes, &edges);

        // Nodes should have moved apart
        let dist = ((nodes[1].position.x - nodes[0].position.x).powi(2)
            + (nodes[1].position.y - nodes[0].position.y).powi(2))
        .sqrt();
        assert!(dist > 50.0); // Should be separated by at least 50
    }

    #[test]
    fn test_route_edges() {
        let nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = CustomLayout::default();

        let routed = layout.route_edges(&nodes, &edges);

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].source, 0);
        assert_eq!(routed[0].target, 1);
        assert!(routed[0].path.len() >= 2);
    }

    #[test]
    fn test_full_layout() {
        let nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = CustomLayout::default();

        let result = layout.layout(nodes, edges);

        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
        println!("Nodes: {:?}", result.nodes);
        println!("Edges: {:?}", result.edges);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    fn generate_svg(result: &LayoutResult, filename: &str, show_grid: bool, show_all_ports: bool) {
        use std::collections::HashSet;
        let mut used_ports = HashSet::new();

        for edge in &result.edges {
            let source_node = &result.nodes[edge.source];
            let target_node = &result.nodes[edge.target];

            // Determine side for source
            let dx = target_node.position.x + target_node.size.width / 2.0
                - (source_node.position.x + source_node.size.width / 2.0);
            let dy = target_node.position.y + target_node.size.height / 2.0
                - (source_node.position.y + source_node.size.height / 2.0);
            let side = if dx.abs() > dy.abs() {
                if dx > 0.0 { "right" } else { "left" }
            } else {
                if dy > 0.0 { "bottom" } else { "top" }
            };
            let port_index = match side {
                "left" => 0,
                "right" => 1,
                "top" => 2,
                "bottom" => 3,
                _ => 0,
            };
            used_ports.insert((edge.source, port_index));

            // Determine side for target
            let dx = source_node.position.x + source_node.size.width / 2.0
                - (target_node.position.x + target_node.size.width / 2.0);
            let dy = source_node.position.y + source_node.size.height / 2.0
                - (target_node.position.y + target_node.size.height / 2.0);
            let side = if dx.abs() > dy.abs() {
                if dx > 0.0 { "right" } else { "left" }
            } else {
                if dy > 0.0 { "bottom" } else { "top" }
            };
            let port_index = match side {
                "left" => 0,
                "right" => 1,
                "top" => 2,
                "bottom" => 3,
                _ => 0,
            };
            used_ports.insert((edge.target, port_index));
        }

        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<rect width="100%" height="100%" fill="white"/>
"#,
            result.canvas_width, result.canvas_height
        );

        // Add grid lines if requested
        if show_grid {
            let cell_size = 5.0;
            let num_x_lines = (result.canvas_width / cell_size).ceil() as usize;
            let num_y_lines = (result.canvas_height / cell_size).ceil() as usize;

            // Vertical grid lines
            for i in 0..=num_x_lines {
                let x = i as f64 * cell_size;
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="0" x2="{}" y2="{}" stroke="lightgray" stroke-width="0.5"/>
"#,
                    x, x, result.canvas_height
                ));
            }

            // Horizontal grid lines
            for i in 0..=num_y_lines {
                let y = i as f64 * cell_size;
                svg.push_str(&format!(
                    r#"<line x1="0" y1="{}" x2="{}" y2="{}" stroke="lightgray" stroke-width="0.5"/>
"#,
                    y, result.canvas_width, y
                ));
            }
        }

        for (_node_index, node) in result.nodes.iter().enumerate() {
            let fill_color = node
                .attributes
                .iter()
                .find(|(k, _)| k == "color")
                .map(|(_, v)| v.as_str())
                .unwrap_or("lightblue");

            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="black"/>
<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>
"#,
                node.position.x,
                node.position.y,
                node.size.width,
                node.size.height,
                fill_color,
                node.position.x + node.size.width / 2.0,
                node.position.y + node.size.height / 2.0 + 5.0,
                node.id
            ));
        }

        for edge in &result.edges {
            if edge.path.len() >= 2 {
                let mut path_data = format!("M {} {}", edge.path[0].x, edge.path[0].y);
                for point in &edge.path[1..] {
                    path_data.push_str(&format!(" L {} {}", point.x, point.y));
                }
                svg.push_str(&format!(
                    r#"<path d="{}" stroke="black" stroke-width="2" fill="none"/>
"#,
                    path_data
                ));
            }
        }

        for (node_index, node) in result.nodes.iter().enumerate() {
            for (port_index, port) in node.ports.iter().enumerate() {
                if show_all_ports || used_ports.contains(&(node_index, port_index)) {
                    let fill = match port.port_type {
                        PortType::Input => "lightblue",
                        PortType::Output => "lightcoral",
                    };
                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="black"/>
"#,
                        node.position.x + port.position.x,
                        node.position.y + port.position.y,
                        port.size.width,
                        port.size.height,
                        fill
                    ));
                }
            }
        }

        svg.push_str("</svg>");

        fs::write(filename, svg).expect("Unable to write SVG");
    }

    #[test]
    fn test_layout_set_1() {
        let nodes = vec![
            Node {
                id: "ECU1".to_string(),
                size: Size {
                    width: 120.0,
                    height: 80.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 120.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 55.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 55.0, y: 80.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "ECU2".to_string(),
                size: Size {
                    width: 100.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 45.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 45.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Sensor".to_string(),
                size: Size {
                    width: 80.0,
                    height: 40.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 80.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 35.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 35.0, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ];
        let edges = vec![(0, 1), (1, 2), (0, 2)];

        let layout = CustomLayout {
            min_spacing: 120.0,
            ..Default::default()
        };
        let result = layout.layout(nodes, edges);

        println!("Set 1 Nodes:");
        for node in &result.nodes {
            println!(
                "  {}: size {:?}, position {:?}",
                node.id, node.size, node.position
            );
        }
        println!("Set 1 Edges:");
        for edge in &result.edges {
            println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
        }

        generate_svg(&result, "test_set_1.svg", true, true);
    }

    #[test]
    fn test_layout_set_2() {
        let nodes = vec![
            Node {
                id: "Gateway".to_string(),
                size: Size {
                    width: 150.0,
                    height: 100.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 150.0, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 70.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 70.0, y: 100.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Display".to_string(),
                size: Size {
                    width: 90.0,
                    height: 70.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 90.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 40.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 40.0, y: 70.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Battery".to_string(),
                size: Size {
                    width: 60.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 60.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 25.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 25.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Motor".to_string(),
                size: Size {
                    width: 110.0,
                    height: 90.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 110.0, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 50.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 50.0, y: 90.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ];
        let edges = vec![(0, 1), (0, 2), (0, 3), (1, 3), (2, 3)];

        let layout = CustomLayout {
            spaced_edges: true,
            min_spacing: 120.0,
            ..Default::default()
        };
        let result = layout.layout(nodes, edges);

        println!("Set 2 Nodes:");
        for node in &result.nodes {
            println!(
                "  {}: size {:?}, position {:?}",
                node.id, node.size, node.position
            );
        }
        println!("Set 2 Edges:");
        for edge in &result.edges {
            println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
        }

        generate_svg(&result, "test_set_2.svg", false, false);
    }

    #[test]
    fn test_layout_set_3() {
        let nodes = vec![
            Node {
                id: "ABS".to_string(),
                size: Size {
                    width: 100.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 45.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 45.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "ESP".to_string(),
                size: Size {
                    width: 120.0,
                    height: 80.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 120.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 55.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 55.0, y: 80.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Airbag".to_string(),
                size: Size {
                    width: 90.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 90.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 40.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 40.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Climate".to_string(),
                size: Size {
                    width: 110.0,
                    height: 70.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 110.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 50.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 50.0, y: 70.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Infotainment".to_string(),
                size: Size {
                    width: 140.0,
                    height: 100.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 140.0, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 65.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 65.0, y: 100.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ];
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 4), (0, 4), (1, 3)];

        let layout = CustomLayout {
            allow_diagonals: false,
            min_spacing: 120.0,
            ..Default::default()
        };
        let result = layout.layout(nodes, edges);

        println!("Set 3 Nodes:");
        for node in &result.nodes {
            println!(
                "  {}: size {:?}, position {:?}",
                node.id, node.size, node.position
            );
        }
        println!("Set 3 Edges:");
        for edge in &result.edges {
            println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
        }

        generate_svg(&result, "test_set_3.svg", true, false);
    }

    #[test]
    fn test_layout_set_4() {
        let nodes = vec![
            Node {
                id: "Engine".to_string(),
                size: Size {
                    width: 140.0,
                    height: 90.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 140.0, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 65.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 65.0, y: 90.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Transmission".to_string(),
                size: Size {
                    width: 120.0,
                    height: 70.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 120.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 55.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 55.0, y: 70.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Brakes".to_string(),
                size: Size {
                    width: 100.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 45.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 45.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Steering".to_string(),
                size: Size {
                    width: 110.0,
                    height: 65.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 27.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 110.0, y: 27.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 50.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 50.0, y: 65.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Sensors".to_string(),
                size: Size {
                    width: 90.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 90.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 40.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 40.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Dashboard".to_string(),
                size: Size {
                    width: 130.0,
                    height: 75.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 32.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 130.0, y: 32.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 60.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 60.0, y: 75.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "ECU".to_string(),
                size: Size {
                    width: 100.0,
                    height: 55.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 45.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 45.0, y: 55.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "FuelPump".to_string(),
                size: Size {
                    width: 85.0,
                    height: 45.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 17.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 85.0, y: 17.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 37.5, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 37.5, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Alternator".to_string(),
                size: Size {
                    width: 95.0,
                    height: 55.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 95.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 42.5, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 42.5, y: 55.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Radiator".to_string(),
                size: Size {
                    width: 110.0,
                    height: 70.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 110.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 50.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 50.0, y: 70.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Battery".to_string(),
                size: Size {
                    width: 80.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 80.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 35.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 35.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "AirFilter".to_string(),
                size: Size {
                    width: 75.0,
                    height: 40.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 75.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 32.5, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 32.5, y: 40.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Exhaust".to_string(),
                size: Size {
                    width: 125.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 125.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 57.5, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 57.5, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            Node {
                id: "Catalytic".to_string(),
                size: Size {
                    width: 105.0,
                    height: 45.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 17.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 105.0, y: 17.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 47.5, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 47.5, y: 45.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ];
        // Complex connectivity with cross-connections and potential routing challenges
        let edges = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6), // main chain
            (0, 3),
            (1, 4),
            (2, 5),
            (0, 6), // cross connections
            (3, 6),
            (4, 6), // additional connections to ECU
            // New component connections
            (0, 7),
            (7, 8),
            (8, 9),
            (9, 10), // fuel system chain
            (1, 11),
            (11, 12),
            (12, 13), // electrical system chain
            (2, 9),
            (3, 10),
            (4, 11), // cooling system connections
            (5, 12),
            (6, 13), // exhaust system connections
            (7, 10),
            (8, 13),
            (9, 6), // additional cross-connections
        ];

        let layout = CustomLayout {
            allow_diagonals: false,
            min_spacing: 120.0,
            ..Default::default()
        };
        let result = layout.layout(nodes, edges);

        println!("Set 4 Nodes:");
        for node in &result.nodes {
            println!(
                "  {}: size {:?}, position {:?}",
                node.id, node.size, node.position
            );
        }
        println!("Set 4 Edges:");
        for edge in &result.edges {
            println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
        }

        generate_svg(&result, "test_set_4.svg", true, false);
    }

    #[test]
    fn test_layout_set_5() {
        let nodes = vec![
            // CAN Bus node with 8 ports
            Node {
                id: "CAN_Bus".to_string(),
                size: Size {
                    width: 200.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    // Left side inputs (2 ports)
                    Port {
                        position: Position { x: -10.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: -10.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    // Right side outputs (2 ports)
                    Port {
                        position: Position { x: 200.0, y: 15.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 200.0, y: 35.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    // Top side inputs (2 ports)
                    Port {
                        position: Position { x: 45.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 145.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    // Bottom side outputs (2 ports)
                    Port {
                        position: Position { x: 45.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 145.0, y: 60.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![("color".to_string(), "grey".to_string())],
            },
            // Ethernet Backbone node with 4 ports
            Node {
                id: "Ethernet_Backbone".to_string(),
                size: Size {
                    width: 180.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    // Left side input
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    // Right side output
                    Port {
                        position: Position { x: 180.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    // Top side input
                    Port {
                        position: Position { x: 85.0, y: -10.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    // Bottom side output
                    Port {
                        position: Position { x: 85.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![("color".to_string(), "grey".to_string())],
            },
            // Gateway ECU connected to both CAN and Ethernet
            Node {
                id: "Gateway_ECU".to_string(),
                size: Size {
                    width: 120.0,
                    height: 80.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: -10.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 120.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                    Port {
                        position: Position { x: 120.0, y: 50.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![("color".to_string(), "violet".to_string())],
            },
            // Engine ECU connected to CAN
            Node {
                id: "Engine_ECU".to_string(),
                size: Size {
                    width: 100.0,
                    height: 60.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            // Transmission ECU connected to CAN
            Node {
                id: "Transmission_ECU".to_string(),
                size: Size {
                    width: 130.0,
                    height: 70.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 130.0, y: 30.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            // Body Control ECU connected to CAN
            Node {
                id: "Body_ECU".to_string(),
                size: Size {
                    width: 110.0,
                    height: 65.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 27.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 110.0, y: 27.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            // Telematics ECU connected to Ethernet
            Node {
                id: "Telematics_ECU".to_string(),
                size: Size {
                    width: 120.0,
                    height: 55.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 120.0, y: 22.5 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
            // ADAS ECU connected to Ethernet
            Node {
                id: "ADAS_ECU".to_string(),
                size: Size {
                    width: 100.0,
                    height: 50.0,
                },
                position: Position { x: 0.0, y: 0.0 },
                ports: vec![
                    Port {
                        position: Position { x: -10.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Input,
                    },
                    Port {
                        position: Position { x: 100.0, y: 20.0 },
                        size: Size {
                            width: 10.0,
                            height: 10.0,
                        },
                        port_type: PortType::Output,
                    },
                ],
                attributes: vec![],
            },
        ];

        // Less interconnected than set 4 - focused on bus topology
        let edges = vec![
            // Gateway ECU connections to CAN Bus (2 connections)
            (0, 2),
            (2, 0),
            // Gateway ECU connections to Ethernet Backbone (2 connections)
            (1, 2),
            (2, 1),
            // CAN-connected ECUs (3 ECUs connected to CAN Bus)
            (0, 3),
            (3, 0), // Engine ECU
            (0, 4),
            (4, 0), // Transmission ECU
            (0, 5),
            (5, 0), // Body ECU
            // Ethernet-connected ECUs (2 ECUs connected to Ethernet Backbone)
            (1, 6),
            (6, 1), // Telematics ECU
            (1, 7),
            (7, 1), // ADAS ECU
        ];

        let layout = CustomLayout {
            allow_diagonals: false,
            min_spacing: 120.0,
            ..Default::default()
        };
        let result = layout.layout(nodes, edges);

        println!("Set 5 Nodes:");
        for node in &result.nodes {
            println!(
                "  {}: size {:?}, position {:?}",
                node.id, node.size, node.position
            );
        }
        println!("Set 5 Edges:");
        for edge in &result.edges {
            println!("  {} -> {}: {:?}", edge.source, edge.target, edge.path);
        }

        generate_svg(&result, "test_set_5.svg", true, true);
    }
}
