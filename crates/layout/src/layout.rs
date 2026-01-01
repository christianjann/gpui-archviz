use crate::grid::*;
use crate::types::*;
use tracing::{Level, debug, enabled};

/// In-place layout method that modifies existing data
pub fn layout_in_place<N: LayoutNode, E: LayoutEdge>(
    nodes: &mut [N],
    edges: &mut [E],
    config: &ArchVizLayout,
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

    let internal_edges: Vec<(usize, usize, Option<usize>, Option<usize>)> = edges
        .iter()
        .map(|e| (e.source(), e.target(), e.source_port(), e.target_port()))
        .collect();

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

impl ArchVizLayout {
    pub fn layout(
        &self,
        nodes: Vec<Node>,
        edges: Vec<(usize, usize, Option<usize>, Option<usize>)>,
    ) -> LayoutResult {
        let mut nodes = nodes;

        // Precompute effective bounds and sizes for performance
        let effective_bounds: Vec<(f64, f64, f64, f64)> =
            nodes.iter().map(|n| n.effective_bounds()).collect();
        let effective_sizes: Vec<Size> = effective_bounds
            .iter()
            .map(|&(min_x, max_x, min_y, max_y)| Size {
                width: max_x - min_x,
                height: max_y - min_y,
            })
            .collect();

        // Dynamically adjust min_spacing based on node sizes to ensure routing space
        let routing_clearance = 15.0; // Minimum space needed between effective bounding boxes for routing

        let adjusted_min_spacing = f64::max(routing_clearance, self.min_spacing);

        // Create a modified config with adjusted spacing
        let config = ArchVizLayout {
            min_spacing: adjusted_min_spacing,
            ..*self
        };

        // Phase 1: Initial placement
        config.initial_placement(&mut nodes, &edges);

        // Phase 2a: Force-directed refinement with initial spacing (no overlap prevention)
        config.force_directed_with_spacing(
            &mut nodes,
            &edges,
            config.initial_spacing,
            false,
            &effective_sizes,
            &effective_bounds,
        );

        // Check for overlaps after initial refinement
        let has_overlaps = config.has_overlaps(&nodes, &effective_bounds);

        if enabled!(Level::DEBUG) {
            if has_overlaps {
                debug!(
                    "OVERLAP DETECTION: Overlaps found after initial refinement, running overlap prevention phase"
                );
            } else {
                debug!("OVERLAP DETECTION: No overlaps after initial refinement");
            }
        }

        // Phase 2b: If overlaps exist, run force-directed refinement with min spacing and overlap prevention
        if has_overlaps {
            config.force_directed_with_spacing(
                &mut nodes,
                &edges,
                config.min_spacing,
                true,
                &effective_sizes,
                &effective_bounds,
            );
        }

        if enabled!(Level::DEBUG) {
            config.detect_overlaps(&nodes, &effective_bounds, &effective_sizes);
        }

        // Phase 3: Edge routing
        let (mut routed_edges, mut grid) = config.route_edges(&mut nodes, &edges);

        // Phase 4: Calculate canvas and center layout
        let (canvas_width, canvas_height) = config.calculate_canvas_size(&nodes, &routed_edges);
        config.center_layout(
            &mut nodes,
            &mut routed_edges,
            canvas_width,
            canvas_height,
            Some(&mut grid),
        );

        LayoutResult {
            nodes,
            edges: routed_edges,
            canvas_width,
            canvas_height,
            grid: Some(grid),
        }
    }

    /// Route edges only, leaving node positions unchanged
    pub fn route_edges_only(
        &self,
        nodes: &[Node],
        edges: &[(usize, usize, Option<usize>, Option<usize>)],
    ) -> Vec<Edge> {
        let mut nodes = nodes.to_vec();
        let (edges, _) = self.route_edges(&mut nodes, edges);
        edges
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

        let internal_edges: Vec<(usize, usize, Option<usize>, Option<usize>)> = edges
            .iter()
            .map(|e| (e.source(), e.target(), e.source_port(), e.target_port()))
            .collect();

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

    fn initial_placement(
        &self,
        nodes: &mut [Node],
        edges: &[(usize, usize, Option<usize>, Option<usize>)],
    ) {
        // Improved clustering based on connectivity with size-aware placement
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; nodes.len()];

        for i in 0..nodes.len() {
            if assigned[i] {
                continue;
            }
            let mut cluster = vec![i];
            assigned[i] = true;

            // Find connected nodes
            for &(a, b, _, _) in edges {
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

        // Place clusters with guaranteed non-overlapping spacing
        let mut placed_clusters: Vec<(f64, f64, f64, f64)> = Vec::new(); // (x, y, width, height)
        let cluster_spacing = self.initial_spacing * 5.0; // Use initial_spacing scaled up for cluster separation

        // First pass: calculate maximum cluster dimensions
        let mut max_cluster_width = 0.0;
        let mut max_cluster_height = 0.0;

        for cluster in &clusters {
            let mut total_height = 0.0;
            let mut max_width = 0.0;

            for &node_idx in cluster {
                max_width = f64::max(max_width, nodes[node_idx].size.width);
                total_height += nodes[node_idx].size.height + self.min_spacing;
            }
            total_height -= self.min_spacing; // Remove last spacing

            max_cluster_width = f64::max(max_cluster_width, max_width);
            max_cluster_height = f64::max(max_cluster_height, total_height);
        }

        // Second pass: place clusters using fixed spacing
        for (i, cluster) in clusters.iter().enumerate() {
            // Place cluster in a grid pattern with fixed large spacing
            let clusters_per_row = 3; // Limit clusters per row
            let row = i / clusters_per_row;
            let col = i % clusters_per_row;

            let cluster_x = col as f64 * cluster_spacing;
            let cluster_y = row as f64 * cluster_spacing;

            // Place the cluster
            let mut y_offset = cluster_y;
            for &node_idx in cluster {
                nodes[node_idx].position = Position {
                    x: cluster_x,
                    y: y_offset,
                };
                y_offset += nodes[node_idx].size.height + self.min_spacing;
            }

            placed_clusters.push((cluster_x, cluster_y, max_cluster_width, max_cluster_height));
        }
    }

    fn force_directed_with_spacing(
        &self,
        nodes: &mut [Node],
        edges: &[(usize, usize, Option<usize>, Option<usize>)],
        spacing: f64,
        prevent_overlaps: bool,
        effective_sizes: &[Size],
        effective_bounds: &[(f64, f64, f64, f64)],
    ) {
        for _ in 0..self.iterations {
            let mut forces: Vec<Position> = vec![Position { x: 0.0, y: 0.0 }; nodes.len()];

            // Calculate repulsion
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let dx = (nodes[j].position.x + effective_sizes[j].width / 2.0)
                        - (nodes[i].position.x + effective_sizes[i].width / 2.0);
                    let dy = (nodes[j].position.y + effective_sizes[j].height / 2.0)
                        - (nodes[i].position.y + effective_sizes[i].height / 2.0);
                    let min_dist = ((effective_sizes[i].width + effective_sizes[j].width) / 2.0
                        + spacing)
                        .max(
                            (effective_sizes[i].height + effective_sizes[j].height) / 2.0 + spacing,
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
            for &(a, b, _, _) in edges {
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
                if prevent_overlaps {
                    let new_x = nodes[i].position.x + forces[i].x * 0.1;
                    let new_y = nodes[i].position.y + forces[i].y * 0.1;

                    // Check if this move would cause any overlaps
                    let mut can_move = true;
                    for j in 0..nodes.len() {
                        if i == j {
                            continue;
                        }

                        let (min_x_i, max_x_i, min_y_i, max_y_i) = effective_bounds[i];
                        let left_i = new_x + min_x_i;
                        let right_i = new_x + max_x_i;
                        let top_i = new_y + min_y_i;
                        let bottom_i = new_y + max_y_i;

                        let (min_x_j, max_x_j, min_y_j, max_y_j) = effective_bounds[j];
                        let left_j = nodes[j].position.x + min_x_j;
                        let right_j = nodes[j].position.x + max_x_j;
                        let top_j = nodes[j].position.y + min_y_j;
                        let bottom_j = nodes[j].position.y + max_y_j;

                        let overlap_x = left_i < right_j && right_i > left_j;
                        let overlap_y = top_i < bottom_j && bottom_i > top_j;

                        if overlap_x && overlap_y {
                            can_move = false;
                            break;
                        }
                    }

                    // Only apply the move if it doesn't cause overlaps
                    if can_move {
                        nodes[i].position.x = new_x;
                        nodes[i].position.y = new_y;
                    }
                } else {
                    // Apply forces directly without overlap prevention
                    nodes[i].position.x += forces[i].x * 0.1;
                    nodes[i].position.y += forces[i].y * 0.1;
                }
            }
        }
    }

    fn detect_overlaps(
        &self,
        nodes: &[Node],
        effective_bounds: &[(f64, f64, f64, f64)],
        effective_sizes: &[Size],
    ) {
        let mut overlaps = Vec::new();

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let node1 = &nodes[i];
                let node2 = &nodes[j];

                // Check if effective bounding boxes overlap
                let (min_x1, max_x1, min_y1, max_y1) = effective_bounds[i];
                let left1 = node1.position.x + min_x1;
                let right1 = node1.position.x + max_x1;
                let top1 = node1.position.y + min_y1;
                let bottom1 = node1.position.y + max_y1;

                let (min_x2, max_x2, min_y2, max_y2) = effective_bounds[j];
                let left2 = node2.position.x + min_x2;
                let right2 = node2.position.x + max_x2;
                let top2 = node2.position.y + min_y2;
                let bottom2 = node2.position.y + max_y2;

                let overlap_x = left1 < right2 && right1 > left2;
                let overlap_y = top1 < bottom2 && bottom1 > top2;

                if overlap_x && overlap_y {
                    overlaps.push((i, j, node1.id.clone(), node2.id.clone()));
                }
            }
        }

        if enabled!(Level::DEBUG) {
            if !overlaps.is_empty() {
                debug!(
                    "OVERLAP DETECTION: Found {} overlapping node pairs:",
                    overlaps.len()
                );
                for (i, j, id1, id2) in overlaps {
                    let node1 = &nodes[i];
                    let node2 = &nodes[j];
                    debug!(
                        "  {} ({:.1},{:.1} effective size {:.1}x{:.1}) overlaps with {} ({:.1},{:.1} effective size {:.1}x{:.1})",
                        id1,
                        node1.position.x,
                        node1.position.y,
                        effective_sizes[i].width,
                        effective_sizes[i].height,
                        id2,
                        node2.position.x,
                        node2.position.y,
                        effective_sizes[j].width,
                        effective_sizes[j].height
                    );
                }
            } else {
                debug!("OVERLAP DETECTION: No overlapping nodes found");
            }
        }
    }

    fn has_overlaps(&self, nodes: &[Node], effective_bounds: &[(f64, f64, f64, f64)]) -> bool {
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let node1 = &nodes[i];
                let node2 = &nodes[j];

                // Check if effective bounding boxes overlap
                let (min_x1, max_x1, min_y1, max_y1) = effective_bounds[i];
                let left1 = node1.position.x + min_x1;
                let right1 = node1.position.x + max_x1;
                let top1 = node1.position.y + min_y1;
                let bottom1 = node1.position.y + max_y1;

                let (min_x2, max_x2, min_y2, max_y2) = effective_bounds[j];
                let left2 = node2.position.x + min_x2;
                let right2 = node2.position.x + max_x2;
                let top2 = node2.position.y + min_y2;
                let bottom2 = node2.position.y + max_y2;

                let overlap_x = left1 < right2 && right1 > left2;
                let overlap_y = top1 < bottom2 && bottom1 > top2;

                if overlap_x && overlap_y {
                    return true;
                }
            }
        }
        false
    }

    /// Calculates the optimal extension distance from a port to the nearest non-obstacle cell.
    /// This ensures ports extend just enough to reach clear routing space without over-extending
    /// into forbidden areas, which would cause ugly routing artifacts.
    fn calculate_extension_distance(
        &self,
        grid: &Grid,
        start_x: usize,
        start_y: usize,
        direction: &str,
        cell_size: f64,
    ) -> f64 {
        match direction {
            "right" => {
                for x in (start_x + 1)..grid.width {
                    if !grid.obstacles[start_y][x] {
                        return (x - start_x) as f64 * cell_size;
                    }
                }
                25.0 // fallback
            }
            "left" => {
                for x in (0..start_x).rev() {
                    if !grid.obstacles[start_y][x] {
                        return (start_x - x) as f64 * cell_size;
                    }
                }
                25.0
            }
            "bottom" => {
                for y in (start_y + 1)..grid.height {
                    if !grid.obstacles[y][start_x] {
                        return (y - start_y) as f64 * cell_size;
                    }
                }
                25.0
            }
            "top" => {
                for y in (0..start_y).rev() {
                    if !grid.obstacles[y][start_x] {
                        return (start_y - y) as f64 * cell_size;
                    }
                }
                25.0
            }
            _ => 25.0,
        }
    }

    fn route_edges(
        &self,
        nodes: &mut [Node],
        edges: &[(usize, usize, Option<usize>, Option<usize>)],
    ) -> (Vec<Edge>, Grid) {
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
        for &(source, target, source_port_id, target_port_id) in edges {
            // Skip self-loops for now
            if source == target {
                continue;
            }

            // Handle source port
            let (source_pos, actual_source_port) = {
                let source_node = &nodes[source];
                if let Some(port_id) = source_port_id {
                    // Check if port with this id exists
                    if let Some(port_index) =
                        source_node.ports.iter().position(|p| p.id == Some(port_id))
                    {
                        let port = &source_node.ports[port_index];
                        (
                            Position {
                                x: source_node.position.x + port.position.x + port.size.width / 2.0,
                                y: source_node.position.y
                                    + port.position.y
                                    + port.size.height / 2.0,
                            },
                            port_index,
                        )
                    } else {
                        // Find best available port (no id assigned yet)
                        let available_ports: Vec<usize> = source_node
                            .ports
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.id.is_none())
                            .map(|(i, _)| i)
                            .collect();
                        if !available_ports.is_empty() {
                            // Select best from available
                            let best_index = available_ports
                                .into_iter()
                                .min_by_key(|&i| {
                                    let port = &source_node.ports[i];
                                    let port_world_x = source_node.position.x
                                        + port.position.x
                                        + port.size.width / 2.0;
                                    let port_world_y = source_node.position.y
                                        + port.position.y
                                        + port.size.height / 2.0;
                                    let target_center_x =
                                        nodes[target].position.x + nodes[target].size.width / 2.0;
                                    let target_center_y =
                                        nodes[target].position.y + nodes[target].size.height / 2.0;
                                    let dx = port_world_x - target_center_x;
                                    let dy = port_world_y - target_center_y;
                                    (dx * dx + dy * dy) as i64
                                })
                                .unwrap();
                            // Assign the id
                            nodes[source].ports[best_index].id = Some(port_id);
                            let port = &nodes[source].ports[best_index];
                            (
                                Position {
                                    x: nodes[source].position.x
                                        + port.position.x
                                        + port.size.width / 2.0,
                                    y: nodes[source].position.y
                                        + port.position.y
                                        + port.size.height / 2.0,
                                },
                                best_index,
                            )
                        } else {
                            // No available ports, fall back to select_port
                            let (pos, opt_port) =
                                self.select_port(&nodes[source], &nodes[target], true);
                            (pos, opt_port.unwrap_or(0))
                        }
                    }
                } else {
                    // No port specified, select best
                    let (pos, opt_port) = self.select_port(&nodes[source], &nodes[target], true);
                    (pos, opt_port.unwrap_or(0))
                }
            };

            // Handle target port
            let (target_pos, actual_target_port) = {
                let target_node = &nodes[target];
                if let Some(port_id) = target_port_id {
                    if let Some(port_index) =
                        target_node.ports.iter().position(|p| p.id == Some(port_id))
                    {
                        let port = &target_node.ports[port_index];
                        (
                            Position {
                                x: target_node.position.x + port.position.x + port.size.width / 2.0,
                                y: target_node.position.y
                                    + port.position.y
                                    + port.size.height / 2.0,
                            },
                            port_index,
                        )
                    } else {
                        let available_ports: Vec<usize> = target_node
                            .ports
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| p.id.is_none())
                            .map(|(i, _)| i)
                            .collect();
                        if !available_ports.is_empty() {
                            let best_index = available_ports
                                .into_iter()
                                .min_by_key(|&i| {
                                    let port = &target_node.ports[i];
                                    let port_world_x = target_node.position.x
                                        + port.position.x
                                        + port.size.width / 2.0;
                                    let port_world_y = target_node.position.y
                                        + port.position.y
                                        + port.size.height / 2.0;
                                    let source_center_x =
                                        nodes[source].position.x + nodes[source].size.width / 2.0;
                                    let source_center_y =
                                        nodes[source].position.y + nodes[source].size.height / 2.0;
                                    let dx = port_world_x - source_center_x;
                                    let dy = port_world_y - source_center_y;
                                    (dx * dx + dy * dy) as i64
                                })
                                .unwrap();
                            nodes[target].ports[best_index].id = Some(port_id);
                            let port = &nodes[target].ports[best_index];
                            (
                                Position {
                                    x: nodes[target].position.x
                                        + port.position.x
                                        + port.size.width / 2.0,
                                    y: nodes[target].position.y
                                        + port.position.y
                                        + port.size.height / 2.0,
                                },
                                best_index,
                            )
                        } else {
                            let (pos, opt_port) =
                                self.select_port(&nodes[target], &nodes[source], false);
                            (pos, opt_port.unwrap_or(0))
                        }
                    }
                } else {
                    let (pos, opt_port) = self.select_port(&nodes[target], &nodes[source], false);
                    (pos, opt_port.unwrap_or(0))
                }
            };

            let source_node = &nodes[source];
            let target_node = &nodes[target];

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

            let source_grid = grid.pos_to_grid(&source_grid_pos);
            let source_extension = self.calculate_extension_distance(
                &grid,
                source_grid.0,
                source_grid.1,
                source_side,
                cell_size,
            );

            let source_ext_x = source_grid_pos.x
                + match source_side {
                    "right" => source_extension,
                    "left" => -source_extension,
                    _ => 0.0,
                };
            let source_ext_y = source_grid_pos.y
                + match source_side {
                    "bottom" => source_extension,
                    "top" => -source_extension,
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

            let target_grid = grid.pos_to_grid(&target_grid_pos);
            let target_extension = self.calculate_extension_distance(
                &grid,
                target_grid.0,
                target_grid.1,
                target_side,
                cell_size,
            );

            let target_ext_x = target_grid_pos.x
                + match target_side {
                    "right" => target_extension,
                    "left" => -target_extension,
                    _ => 0.0,
                };
            let target_ext_y = target_grid_pos.y
                + match target_side {
                    "bottom" => target_extension,
                    "top" => -target_extension,
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

            if enabled!(Level::DEBUG) {
                // Debug: check for diagonal segments
                for i in 1..full_path.len() {
                    let p1 = &full_path[i - 1];
                    let p2 = &full_path[i];
                    if p1.x != p2.x && p1.y != p2.y {
                        debug!(
                            "WARNING: Diagonal segment calculated: ({}, {}) to ({}, {})",
                            p1.x, p1.y, p2.x, p2.y
                        );
                    }
                }
            }

            routed_edges.push(Edge {
                source,
                target,
                source_port: Some(actual_source_port),
                target_port: Some(actual_target_port),
                path: full_path,
            });
        }

        (routed_edges, grid)
    }

    fn select_port(
        &self,
        from_node: &Node,
        to_node: &Node,
        _prefer_output: bool,
    ) -> (Position, Option<usize>) {
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
            (
                Position {
                    x: snapped_x,
                    y: snapped_y,
                },
                None,
            )
        } else {
            // Calculate direction vector from from_node center to to_node center
            let from_center_x = from_node.position.x + from_node.size.width / 2.0;
            let from_center_y = from_node.position.y + from_node.size.height / 2.0;
            let to_center_x = to_node.position.x + to_node.size.width / 2.0;
            let to_center_y = to_node.position.y + to_node.size.height / 2.0;

            let dx = to_center_x - from_center_x;
            let dy = to_center_y - from_center_y;

            // Find the port whose position best matches the direction
            let mut best_port_index = None;
            let mut best_score = f64::INFINITY;

            for (i, port) in from_node.ports.iter().enumerate() {
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
                        best_port_index = Some(i);
                    }
                }
            }

            if let Some(port_index) = best_port_index {
                let port = &from_node.ports[port_index];
                (
                    Position {
                        x: from_node.position.x + port.position.x + port.size.width / 2.0,
                        y: from_node.position.y + port.position.y + port.size.height / 2.0,
                    },
                    Some(port_index),
                )
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
                (
                    Position {
                        x: snapped_x,
                        y: snapped_y,
                    },
                    None,
                )
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
        grid: Option<&mut Grid>,
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

        // Update grid origin to match the centered coordinate system
        if let Some(grid) = grid {
            grid.origin_x += offset_x;
            grid.origin_y += offset_y;
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
                        id: None,
                    }, // left
                    Port {
                        position: Position { x: 100.0, y: 25.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                        id: None,
                    }, // right
                    Port {
                        position: Position { x: 50.0, y: 0.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                        id: None,
                    }, // top
                    Port {
                        position: Position { x: 50.0, y: 50.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                        id: None,
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
                        id: None,
                    },
                    Port {
                        position: Position { x: 80.0, y: 30.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                        id: None,
                    },
                    Port {
                        position: Position { x: 40.0, y: 0.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Input,
                        id: None,
                    },
                    Port {
                        position: Position { x: 40.0, y: 60.0 },
                        size: Size {
                            width: 50.0,
                            height: 50.0,
                        },
                        port_type: PortType::Output,
                        id: None,
                    },
                ],
                attributes: vec![],
            },
        ]
    }

    fn create_test_edge_indices() -> Vec<(usize, usize, Option<usize>, Option<usize>)> {
        vec![(0, 1, None, None)]
    }

    #[test]
    fn test_initial_placement() {
        let mut nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = ArchVizLayout::default();

        layout.initial_placement(&mut nodes, &edges);

        assert!(nodes[0].position.x >= 0.0);
        assert!(nodes[1].position.x >= 0.0);
    }

    #[test]
    fn test_force_directed() {
        let mut nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = ArchVizLayout {
            iterations: 200,
            repulsion_strength: 10000.0,
            ..Default::default()
        };

        let effective_bounds: Vec<(f64, f64, f64, f64)> =
            nodes.iter().map(|n| n.effective_bounds()).collect();
        let effective_sizes: Vec<Size> = effective_bounds
            .iter()
            .map(|&(min_x, max_x, min_y, max_y)| Size {
                width: max_x - min_x,
                height: max_y - min_y,
            })
            .collect();

        layout.force_directed_with_spacing(
            &mut nodes,
            &edges,
            layout.min_spacing,
            true,
            &effective_sizes,
            &effective_bounds,
        );

        // Nodes should have moved apart
        let dist = ((nodes[1].position.x - nodes[0].position.x).powi(2)
            + (nodes[1].position.y - nodes[0].position.y).powi(2))
        .sqrt();
        assert!(dist > 50.0); // Should be separated by at least 50
    }

    #[test]
    fn test_route_edges() {
        let mut nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = ArchVizLayout::default();

        let (routed, _) = layout.route_edges(&mut nodes, &edges);

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].source, 0);
        assert_eq!(routed[0].target, 1);
        assert!(routed[0].path.len() >= 2);
    }

    #[test]
    fn test_full_layout() {
        let nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = ArchVizLayout::default();

        let result = layout.layout(nodes, edges);

        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
        println!("Nodes: {:?}", result.nodes);
        println!("Edges: {:?}", result.edges);
    }

    #[test]
    fn test_route_edges_only() {
        let nodes = create_test_nodes();
        let edges = create_test_edge_indices();
        let layout = ArchVizLayout::default();

        let routed_edges = layout.route_edges_only(&nodes, &edges);

        assert_eq!(routed_edges.len(), 1);
        assert_eq!(routed_edges[0].source, 0);
        assert_eq!(routed_edges[0].target, 1);
        assert!(routed_edges[0].path.len() >= 2);
    }
}
