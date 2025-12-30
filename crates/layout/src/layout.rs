use crate::grid::*;
use crate::types::*;

/// In-place layout method that modifies existing data
pub fn layout_in_place<N: LayoutNode, E: LayoutEdge>(
    nodes: &mut [N],
    edges: &mut [E],
    config: &CustomLayout,
) -> Result<(), LayoutError> {
    // Convert to internal format
    let internal_nodes: Vec<Node> = nodes
        .iter()
        .map(|n| Node {
            id: n.id(),
            position: n.position(),
            size: n.size(),
            ports: n.ports(),
            attributes: vec![],
        })
        .collect();

    let internal_edges: Vec<(usize, usize)> =
        edges.iter().map(|e| (e.source(), e.target())).collect();

    // Run layout
    let result = config.layout(internal_nodes, internal_edges);

    // Update original data
    for (i, node) in result.nodes.into_iter().enumerate() {
        nodes[i].set_position(node.position);
    }

    for (i, edge) in result.edges.into_iter().enumerate() {
        edges[i].set_path(edge.path);
    }

    Ok(())
}

impl CustomLayout {
    pub fn layout(&self, nodes: Vec<Node>, edges: Vec<(usize, usize)>) -> LayoutResult {
        let mut nodes = nodes;

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

    /// In-place layout method that modifies existing data structures
    pub fn layout_in_place<N: LayoutNode, E: LayoutEdge>(
        &self,
        nodes: &mut [N],
        edges: &mut [E],
    ) -> Result<(), LayoutError> {
        // Convert to internal format
        let internal_nodes: Vec<Node> = nodes
            .iter()
            .map(|n| Node {
                id: n.id(),
                position: n.position(),
                size: n.size(),
                ports: n.ports(),
                attributes: vec![],
            })
            .collect();

        let internal_edges: Vec<(usize, usize)> =
            edges.iter().map(|e| (e.source(), e.target())).collect();

        // Run layout
        let result = self.layout(internal_nodes, internal_edges);

        // Update original data in-place
        for (i, node) in result.nodes.into_iter().enumerate() {
            if i < nodes.len() {
                nodes[i].set_position(node.position);
            } else {
                return Err(LayoutError::InvalidNodeIndex);
            }
        }

        for (i, edge) in result.edges.into_iter().enumerate() {
            if i < edges.len() {
                edges[i].set_path(edge.path);
            } else {
                return Err(LayoutError::InvalidEdgeIndex);
            }
        }

        Ok(())
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
            } else if (source_pos.x - (source_node.position.x + source_node.size.width)).abs()
                < 10.0
            {
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

            let source_ext_end =
                nearest_grid(&source_ext_end, grid.origin_x, grid.origin_y, cell_size);

            // Calculate direction for target based on port side
            let target_side = if (target_pos.x - target_node.position.x).abs() < 10.0 {
                "left"
            } else if (target_pos.x - (target_node.position.x + target_node.size.width)).abs()
                < 10.0
            {
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

            let target_ext_end =
                nearest_grid(&target_ext_end, grid.origin_x, grid.origin_y, cell_size);

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
                let p1 = &full_path[i - 1];
                let p2 = &full_path[i];
                if p1.x != p2.x && p1.y != p2.y {
                    println!(
                        "WARNING: Diagonal segment calculated: ({}, {}) to ({}, {})",
                        p1.x, p1.y, p2.x, p2.y
                    );
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
        edges: &mut [Edge],
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
