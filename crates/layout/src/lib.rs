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
    pub ports: Vec<Port>, // up to 8 ports
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
    fn new(nodes: &[Node], canvas_width: f64, canvas_height: f64, cell_size: f64) -> Self {
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
            let x1 = ((node.position.x - 2.0 - min_x) / cell_size).floor() as usize;
            let y1 = ((node.position.y - 2.0 - min_y) / cell_size).floor() as usize;
            let x2 =
                ((node.position.x + node.size.width + 2.0 - min_x) / cell_size).ceil() as usize;
            let y2 =
                ((node.position.y + node.size.height + 2.0 - min_y) / cell_size).ceil() as usize;

            for y in y1..y2.min(height) {
                for x in x1..x2.min(width) {
                    obstacles[y][x] = true;
                }
            }
        }

        // Block port areas
        /*
        for node in nodes {
            for port in &node.ports {
                let port_x1 = (((node.position.x + port.position.x - min_x) / cell_size).floor() as i32 - 1).max(0) as usize;
                let port_y1 = (((node.position.y + port.position.y - min_y) / cell_size).floor() as i32 - 1).max(0) as usize;
                let port_x2 = (((node.position.x + port.position.x + port.size.width - min_x) / cell_size).ceil() as i32 + 1).min((width - 1) as i32) as usize + 1;
                let port_y2 = (((node.position.y + port.position.y + port.size.height - min_y) / cell_size).ceil() as i32 + 1).min((height - 1) as i32) as usize + 1;

                for y in port_y1..port_y2.min(height) {
                    for x in port_x1..port_x2.min(width) {
                        obstacles[y][x] = true;
                    }
                }
            }
        }
        */

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
        let x = (((pos.x - self.origin_x) / self.cell_size).floor() as usize).min(self.width - 1);
        let y = (((pos.y - self.origin_y) / self.cell_size).floor() as usize).min(self.height - 1);
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

        let cell_size = 10.0;
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

            // Calculate direction for source
            let dx = target_node.position.x + target_node.size.width / 2.0
                - (source_node.position.x + source_node.size.width / 2.0);
            let dy = target_node.position.y + target_node.size.height / 2.0
                - (source_node.position.y + source_node.size.height / 2.0);
            let source_direction = if dx.abs() > dy.abs() {
                if dx > 0.0 { "right" } else { "left" }
            } else {
                if dy > 0.0 { "bottom" } else { "top" }
            };

            let source_ext_x = source_grid_pos.x
                + match source_direction {
                    "right" => 10.0,
                    "left" => -10.0,
                    _ => 0.0,
                };
            let source_ext_y = source_grid_pos.y
                + match source_direction {
                    "bottom" => 10.0,
                    "top" => -10.0,
                    _ => 0.0,
                };
            let source_ext_end = Position {
                x: source_ext_x,
                y: source_ext_y,
            };

            // Calculate direction for target
            let dx = source_node.position.x + source_node.size.width / 2.0
                - (target_node.position.x + target_node.size.width / 2.0);
            let dy = source_node.position.y + source_node.size.height / 2.0
                - (target_node.position.y + target_node.size.height / 2.0);
            let target_direction = if dx.abs() > dy.abs() {
                if dx > 0.0 { "right" } else { "left" }
            } else {
                if dy > 0.0 { "bottom" } else { "top" }
            };

            let target_ext_x = target_grid_pos.x
                + match target_direction {
                    "right" => 10.0,
                    "left" => -10.0,
                    _ => 0.0,
                };
            let target_ext_y = target_grid_pos.y
                + match target_direction {
                    "bottom" => 10.0,
                    "top" => -10.0,
                    _ => 0.0,
                };
            let target_ext_end = Position {
                x: target_ext_x,
                y: target_ext_y,
            };

            let source_ext_grid = grid.pos_to_grid(&source_ext_end);
            grid.obstacles[source_ext_grid.1][source_ext_grid.0] = false;

            let target_ext_grid = grid.pos_to_grid(&target_ext_end);
            grid.obstacles[target_ext_grid.1][target_ext_grid.0] = false;

            let path = grid.find_path(source_ext_end, target_ext_end);
            let mut full_path = vec![source_grid_pos];
            full_path.extend(path);
            full_path.push(target_grid_pos);

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
            // Select port on side facing the target
            let dx = to_node.position.x + to_node.size.width / 2.0
                - (from_node.position.x + from_node.size.width / 2.0);
            let dy = to_node.position.y + to_node.size.height / 2.0
                - (from_node.position.y + from_node.size.height / 2.0);

            // Determine direction
            let side = if dx.abs() > dy.abs() {
                if dx > 0.0 { "right" } else { "left" }
            } else {
                if dy > 0.0 { "bottom" } else { "top" }
            };

            // Find port on that side
            for port in &from_node.ports {
                // Check exact position on side
                match side {
                    "right" if port.position.x == from_node.size.width => {
                        let port_left = from_node.position.x + port.position.x;
                        let port_top = from_node.position.y + port.position.y;
                        let port_center_x = port_left + port.size.width / 2.0;
                        let port_center_y = port_top + port.size.height / 2.0;
                        return Position {
                            x: port_center_x,
                            y: port_center_y,
                        };
                    }
                    "left" if port.position.x < 0.0 => {
                        let port_left = from_node.position.x + port.position.x;
                        let port_top = from_node.position.y + port.position.y;
                        let port_center_x = port_left + port.size.width / 2.0;
                        let port_center_y = port_top + port.size.height / 2.0;
                        return Position {
                            x: port_center_x,
                            y: port_center_y,
                        };
                    }
                    "bottom" if port.position.y == from_node.size.height => {
                        let port_left = from_node.position.x + port.position.x;
                        let port_top = from_node.position.y + port.position.y;
                        let port_center_x = port_left + port.size.width / 2.0;
                        let port_center_y = port_top + port.size.height / 2.0;
                        return Position {
                            x: port_center_x,
                            y: port_center_y,
                        };
                    }
                    "top" if port.position.y < 0.0 => {
                        let port_left = from_node.position.x + port.position.x;
                        let port_top = from_node.position.y + port.position.y;
                        let port_center_x = port_left + port.size.width / 2.0;
                        let port_center_y = port_top + port.size.height / 2.0;
                        return Position {
                            x: port_center_x,
                            y: port_center_y,
                        };
                    }
                    _ => {}
                }
            }
            // Fallback to center
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

    fn generate_svg(result: &LayoutResult, filename: &str) {
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

        for (node_index, node) in result.nodes.iter().enumerate() {
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="lightblue" stroke="black"/>
<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>
"#,
                node.position.x,
                node.position.y,
                node.size.width,
                node.size.height,
                node.position.x + node.size.width / 2.0,
                node.position.y + node.size.height / 2.0 + 5.0,
                node.id
            ));
            for (port_index, port) in node.ports.iter().enumerate() {
                if used_ports.contains(&(node_index, port_index)) {
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

        generate_svg(&result, "test_set_1.svg");
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

        generate_svg(&result, "test_set_2.svg");
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

        generate_svg(&result, "test_set_3.svg");
    }
}
