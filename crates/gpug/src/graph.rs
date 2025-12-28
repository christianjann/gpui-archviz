use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui::{canvas, div, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::ActiveTheme;

use crate::edge::GpugEdge;
use crate::node::GpugNode;

/// Edge routing style
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EdgeRouting {
    /// Direct straight line between ports
    #[default]
    Straight,
    /// Manhattan-style orthogonal routing (horizontal and vertical segments only)
    Manhattan,
}

pub struct Graph {
    pub nodes: Vec<Entity<GpugNode>>,
    pub edges: Vec<GpugEdge>,
    pub k: usize,
    pub beta: f32,
    pub sim_tick: u64,
    pub zoom: f32,
    pub pan: Point<Pixels>,
    pub playing: bool,
    pub container_offset: Point<Pixels>,
    pub container_size: Size<Pixels>,
    pub needs_layout: bool,
    // For panning with mouse drag
    pub is_panning: bool,
    pub pan_start: Point<Pixels>,
    pub pan_start_pos: Point<Pixels>,
    /// Edge routing style
    pub edge_routing: EdgeRouting,
}

impl Graph {
    pub fn new(
        cx: &mut App,
        nodes: Vec<GpugNode>,
        edges: Vec<GpugEdge>,
        k: usize,
        beta: f32,
    ) -> Self {
        let zoom = 1.0;
        let pan = point(px(0.0), px(0.0));
        let mut node_entities: Vec<Entity<GpugNode>> = Vec::with_capacity(nodes.len());

        for mut node in nodes {
            node.zoom = zoom;
            node.pan = pan;
            node_entities.push(cx.new(|_| node));
        }

        Self {
            nodes: node_entities,
            edges,
            k,
            beta,
            sim_tick: 0,
            zoom,
            pan,
            playing: false,
            container_offset: point(px(0.0), px(0.0)),
            container_size: size(px(0.0), px(0.0)),
            needs_layout: true,
            is_panning: false,
            pan_start: point(px(0.0), px(0.0)),
            pan_start_pos: point(px(0.0), px(0.0)),
            edge_routing: EdgeRouting::default(),
        }
    }

    /// Relayout nodes to fit within the container bounds
    fn layout_nodes(&mut self, cx: &mut Context<Self>) {
        if !self.needs_layout || self.container_size.width <= px(0.0) || self.container_size.height <= px(0.0) {
            return;
        }
        self.needs_layout = false;

        let margin = 60.0f32;
        let width = (self.container_size.width / px(1.0)) as f32 - margin * 2.0;
        let height = (self.container_size.height / px(1.0)) as f32 - margin * 2.0;

        if width <= 0.0 || height <= 0.0 {
            return;
        }

        // Find current bounds of all nodes
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for node in &self.nodes {
            let (x, y) = cx.read_entity(node, |n, _| ((n.x / px(1.0)) as f32, (n.y / px(1.0)) as f32));
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        let current_width = (max_x - min_x).max(1.0);
        let current_height = (max_y - min_y).max(1.0);

        // Scale factor to fit nodes in container
        let scale_x = width / current_width;
        let scale_y = height / current_height;
        let scale = scale_x.min(scale_y);

        // Reposition nodes to fit container
        for node in &self.nodes {
            cx.update_entity(node, |n, _| {
                let nx = (n.x / px(1.0)) as f32;
                let ny = (n.y / px(1.0)) as f32;
                // Normalize to 0-1, then scale to container
                let norm_x = (nx - min_x) / current_width;
                let norm_y = (ny - min_y) / current_height;
                n.x = px(margin + norm_x * width.min(current_width * scale));
                n.y = px(margin + norm_y * height.min(current_height * scale));
            });
        }
    }

    /// Set the edge routing style
    pub fn set_edge_routing(&mut self, routing: EdgeRouting, cx: &mut Context<Self>) {
        if self.edge_routing != routing {
            self.edge_routing = routing;
            cx.notify();
        }
    }

    /// Update the graph with new nodes and edges
    pub fn update_model(&mut self, nodes: Vec<GpugNode>, edges: Vec<GpugEdge>, cx: &mut Context<Self>) {
        // Create new node entities
        let mut node_entities: Vec<Entity<GpugNode>> = Vec::with_capacity(nodes.len());
        for mut node in nodes {
            node.zoom = self.zoom;
            node.pan = self.pan;
            node.container_offset = self.container_offset;
            node_entities.push(cx.new(|_| node));
        }
        
        self.nodes = node_entities;
        self.edges = edges;
        self.needs_layout = true;
        cx.notify();
    }
    
    /// Set zoom level and update all nodes
    pub fn set_zoom(&mut self, new_zoom: f32, cx: &mut Context<Self>) {
        let new_zoom = new_zoom.clamp(0.1, 3.0);
        if (new_zoom - self.zoom).abs() < 0.001 {
            return;
        }
        self.zoom = new_zoom;
        let zoom = self.zoom;
        for n in &self.nodes {
            cx.update_entity(n, move |node, _| {
                node.zoom = zoom;
            });
        }
        cx.notify();
    }
    
    /// Fit all nodes into the visible area
    pub fn fit_to_content(&mut self, cx: &mut Context<Self>) {
        if self.nodes.is_empty() || self.container_size.width <= px(0.0) {
            return;
        }
        
        // Find bounding box of all nodes
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for n in &self.nodes {
            let (x, y, w, h) = cx.read_entity(n, |node, _| {
                let (w, h) = node.estimate_dimensions();
                ((node.x / px(1.0)) as f32, (node.y / px(1.0)) as f32, w, h)
            });
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + w);
            max_y = max_y.max(y + h);
        }
        
        let content_width = max_x - min_x;
        let content_height = max_y - min_y;
        
        if content_width <= 0.0 || content_height <= 0.0 {
            return;
        }
        
        // Calculate zoom to fit with some padding
        let padding = 40.0;
        let available_width = (self.container_size.width / px(1.0)) as f32 - padding * 2.0;
        let available_height = (self.container_size.height / px(1.0)) as f32 - padding * 2.0;
        
        let zoom_x = available_width / content_width;
        let zoom_y = available_height / content_height;
        let new_zoom = zoom_x.min(zoom_y).clamp(0.1, 2.0);
        
        // Center the content
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let container_width = (self.container_size.width / px(1.0)) as f32;
        let container_height = (self.container_size.height / px(1.0)) as f32;
        let pan_x = container_width / 2.0 - center_x * new_zoom;
        let pan_y = container_height / 2.0 - center_y * new_zoom;
        
        self.zoom = new_zoom;
        self.pan = point(px(pan_x), px(pan_y));
        
        // Update all nodes
        let zoom = self.zoom;
        let pan = self.pan;
        for n in &self.nodes {
            cx.update_entity(n, move |node, _| {
                node.zoom = zoom;
                node.pan = pan;
            });
        }
        cx.notify();
    }
}

fn parameter_button<F>(label: &str, text_color: Hsla, border_color: Hsla, cx: &mut Context<Graph>, on_press: F) -> Div
where
    F: Fn(&mut Graph, &mut Context<Graph>) + 'static,
{
    div()
        .child(label.to_string())
        .p(px(4.0))
        .text_color(text_color)
        .border(px(1.0))
        .border_color(border_color)
        .rounded(px(4.0))
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |this, _event: &gpui::MouseDownEvent, _window, cx| {
                on_press(this, cx);
            }),
        )
}

impl Render for Graph {
    fn render(&mut self, _window: &mut Window, graph_cx: &mut Context<Self>) -> impl IntoElement {
        // Capture container bounds and update nodes with offset, trigger layout if needed
        let nodes_for_offset = self.nodes.clone();
        let graph_entity_for_offset = graph_cx.entity();
        let bounds_tracker = canvas(
            |_bounds, _window, _cx| (),
            move |bounds, _state, _window, cx| {
                let offset = bounds.origin;
                let size = bounds.size;
                // Update container_offset/size on Graph and all nodes, trigger layout
                cx.update_entity(&graph_entity_for_offset, |graph, cx| {
                    let offset_changed = graph.container_offset != offset;
                    let size_changed = graph.container_size != size;
                    
                    if offset_changed {
                        graph.container_offset = offset;
                        for node in &nodes_for_offset {
                            cx.update_entity(node, |n, _| {
                                n.container_offset = offset;
                            });
                        }
                    }
                    
                    if size_changed {
                        graph.container_size = size;
                        // Layout nodes to fit the new container size
                        graph.layout_nodes(cx);
                    }
                });
            },
        )
        .absolute()
        .size_full();

        // Batched edges canvas: draw all edges in a single paint pass
        let zoom = self.zoom;
        let pan = self.pan;
        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        let edge_routing = self.edge_routing;
        let edges_canvas = canvas(
            |_bounds, _window, _cx| (),
            move |bounds, _state, window, cx| {
                // Use bounds.origin to offset painting to the container's position
                let offset = bounds.origin;
                let thickness = (1.0f32 * zoom).max(1.0);
                
                // Port positioning constants (must match node.rs)
                let header_height = 28.0f32;
                // Port vertical center is at header_height / 2 from node top
                let port_y_offset = header_height / 2.0;
                
                // Port colors for highlighted edges
                let source_color = rgb(0xff8844); // Orange (outgoing port)
                let target_color = rgb(0x4488ff); // Blue (incoming port)
                let normal_color = rgb(0x323232);
                
                // Helper closure to draw a thick line segment to a path
                let draw_segment = |path: &mut gpui::Path<Pixels>, p1: Point<Pixels>, p2: Point<Pixels>, half_thickness: f32| {
                    let dir = point(p2.x - p1.x, p2.y - p1.y);
                    let len = dir.magnitude() as f32;
                    if len <= 0.0001 {
                        return;
                    }
                    let normal = point(-dir.y, dir.x) * (half_thickness / len);

                    let p1a = point(p1.x + normal.x, p1.y + normal.y);
                    let p1b = point(p1.x - normal.x, p1.y - normal.y);
                    let p2a = point(p2.x + normal.x, p2.y + normal.y);
                    let p2b = point(p2.x - normal.x, p2.y - normal.y);

                    let st = (point(0., 1.), point(0., 1.), point(0., 1.));
                    path.push_triangle((p1a, p1b, p2a), st);
                    path.push_triangle((p2a, p1b, p2b), st);
                };
                
                // Collect edge data with selection state
                #[derive(Clone, Copy)]
                enum EdgeSelection {
                    None,
                    SourceSelected,  // Edge is outgoing from selected node (orange)
                    TargetSelected,  // Edge is incoming to selected node (blue)
                    BothSelected,    // Both nodes selected
                }
                
                struct EdgeData {
                    p1: Point<Pixels>,
                    p2: Point<Pixels>,
                    selection: EdgeSelection,
                }
                
                let mut edge_data: Vec<EdgeData> = Vec::with_capacity(edges.len());
                
                for edge in &edges {
                    let i = edge.source;
                    let j = edge.target;
                    if i >= nodes.len() || j >= nodes.len() {
                        continue;
                    }
                    // Connect from source's right port to target's left port
                    // Right port center: node.x + node.width (port extends past node edge)
                    // Left port center: node.x (port extends before node edge)
                    // Vertical: port_y_offset from top of node
                    let (x1, y1, source_selected) = cx.read_entity(&nodes[i], |n, _| (
                        n.x + px(n.width), // Right port center x
                        n.y + px(port_y_offset), // Port vertical center
                        n.selected
                    ));
                    let (x2, y2, target_selected) = cx.read_entity(&nodes[j], |n, _| (
                        n.x, // Left port center x
                        n.y + px(port_y_offset), // Port vertical center
                        n.selected
                    ));

                    // Offset by bounds.origin so edges are drawn relative to container
                    let p1 = point(offset.x + pan.x + x1 * zoom, offset.y + pan.y + y1 * zoom);
                    let p2 = point(offset.x + pan.x + x2 * zoom, offset.y + pan.y + y2 * zoom);
                    
                    let selection = match (source_selected, target_selected) {
                        (true, true) => EdgeSelection::BothSelected,
                        (true, false) => EdgeSelection::SourceSelected,  // Outgoing from selected
                        (false, true) => EdgeSelection::TargetSelected,  // Incoming to selected
                        (false, false) => EdgeSelection::None,
                    };
                    
                    edge_data.push(EdgeData {
                        p1,
                        p2,
                        selection,
                    });
                }
                
                // Helper to draw edge segments based on routing
                let draw_edge = |path: &mut gpui::Path<Pixels>, p1: Point<Pixels>, p2: Point<Pixels>, half_thickness: f32| {
                    match edge_routing {
                        EdgeRouting::Straight => {
                            draw_segment(path, p1, p2, half_thickness);
                        }
                        EdgeRouting::Manhattan => {
                            let mid_x = (p1.x + p2.x) / 2.0;
                            let corner1 = point(mid_x, p1.y);
                            let corner2 = point(mid_x, p2.y);
                            draw_segment(path, p1, corner1, half_thickness);
                            draw_segment(path, corner1, corner2, half_thickness);
                            draw_segment(path, corner2, p2, half_thickness);
                        }
                    }
                };
                
                // Draw glow for selected edges first (underneath)
                // Orange glow for outgoing, blue glow for incoming
                let mut outgoing_glow_path = gpui::Path::new(offset);
                let mut incoming_glow_path = gpui::Path::new(offset);
                for edge in &edge_data {
                    match edge.selection {
                        EdgeSelection::SourceSelected | EdgeSelection::BothSelected => {
                            draw_edge(&mut outgoing_glow_path, edge.p1, edge.p2, thickness * 4.0);
                        }
                        EdgeSelection::TargetSelected => {
                            draw_edge(&mut incoming_glow_path, edge.p1, edge.p2, thickness * 4.0);
                        }
                        EdgeSelection::None => {}
                    }
                }
                window.paint_path(outgoing_glow_path, rgba(0xff884460)); // Orange glow
                window.paint_path(incoming_glow_path, rgba(0x4488ff60)); // Blue glow
                
                // Draw normal (non-selected) edges
                let mut normal_path = gpui::Path::new(offset);
                for edge in &edge_data {
                    if matches!(edge.selection, EdgeSelection::None) {
                        draw_edge(&mut normal_path, edge.p1, edge.p2, thickness);
                    }
                }
                window.paint_path(normal_path, normal_color);
                
                // Draw selected edges with appropriate colors
                // Orange for outgoing (source selected), blue for incoming (target selected)
                let mut outgoing_path = gpui::Path::new(offset);
                let mut incoming_path = gpui::Path::new(offset);
                for edge in &edge_data {
                    match edge.selection {
                        EdgeSelection::SourceSelected | EdgeSelection::BothSelected => {
                            draw_edge(&mut outgoing_path, edge.p1, edge.p2, thickness * 2.0);
                        }
                        EdgeSelection::TargetSelected => {
                            draw_edge(&mut incoming_path, edge.p1, edge.p2, thickness * 2.0);
                        }
                        EdgeSelection::None => {}
                    }
                }
                window.paint_path(outgoing_path, source_color);  // Orange for outgoing
                window.paint_path(incoming_path, target_color);  // Blue for incoming
            },
        )
        .absolute()
        .size_full();

        // Node entities render above edges
        let graph_canvas = div()
            .relative()
            .size_full()
            .child(bounds_tracker)
            .child(edges_canvas)
            .children(self.nodes.iter().cloned());

        // Get theme colors for controls
        let text_color = graph_cx.theme().foreground;
        let border_color = graph_cx.theme().border;
        let bg_color = graph_cx.theme().secondary;
        
        // Zoom controls panel
        let zoom_percent = (self.zoom * 100.0) as i32;
        let controls_panel = {
            let zoom_out = parameter_button("-", text_color, border_color, graph_cx, |this, cx| {
                this.set_zoom(this.zoom - 0.1, cx);
            });
            let zoom_in = parameter_button("+", text_color, border_color, graph_cx, |this, cx| {
                this.set_zoom(this.zoom + 0.1, cx);
            });
            let fit_button = div()
                .px(px(8.0))
                .py(px(4.0))
                .text_color(text_color)
                .border(px(1.0))
                .border_color(border_color)
                .rounded(px(4.0))
                .cursor_pointer()
                .hover(|this| this.bg(bg_color))
                .child("Fit")
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    graph_cx.listener(|this, _e: &gpui::MouseDownEvent, _w, cx| {
                        this.fit_to_content(cx);
                    }),
                );

            div()
                .absolute()
                .top(px(8.0))
                .left(px(8.0))
                .text_color(text_color)
                .bg(graph_cx.theme().background.opacity(0.9))
                .border(px(1.0))
                .border_color(border_color)
                .rounded(px(6.0))
                .p(px(8.0))
                .flex()
                .items_center()
                .gap_2()
                .child(zoom_out)
                .child(format!("{}%", zoom_percent))
                .child(zoom_in)
                .child(div().w(px(8.0))) // spacer
                .child(fit_button)
        };

        // Simulation canvas: runs a physics step per frame when playing
        let graph_entity = graph_cx.entity();
        let graph_handle = graph_entity.clone();
        let nodes_for_sim = self.nodes.clone();
        let edges = self.edges.clone();
        let sim_canvas = canvas(
            move |_bounds, _window, _cx| (),
            move |_bounds, _state, window, cx| {
                let playing = cx.read_entity(&graph_handle, |g: &Graph, _| g.playing);
                if !playing {
                    return;
                }
                let n = nodes_for_sim.len();
                if n == 0 {
                    return;
                }

                window.request_animation_frame();

                // Read positions and sizes
                let mut xs: Vec<f32> = Vec::with_capacity(n);
                let mut ys: Vec<f32> = Vec::with_capacity(n);
                let mut widths: Vec<f32> = Vec::with_capacity(n);
                let mut heights: Vec<f32> = Vec::with_capacity(n);
                for ent in &nodes_for_sim {
                    let (x, y, w, h) = cx.read_entity(ent, |nd, _| (nd.x, nd.y, nd.width, nd.height));
                    xs.push((x / px(1.0)) as f32);
                    ys.push((y / px(1.0)) as f32);
                    widths.push(w);
                    heights.push(h);
                }

                let mut fx = vec![0.0f32; n];
                let mut fy = vec![0.0f32; n];

                // Force parameters (tune for stability/perf)
                // Increased repulsion for larger rectangular nodes (80x32)
                let repulsion = 800.0f32; // higher repulsion for more spacing
                let attraction = 0.02f32; // slightly weaker springs
                let gravity = 0.004f32; // pull toward center
                let damping = 0.85f32; // velocity damping
                let dt = 0.5f32; // larger step, clamped below
                let max_disp = 8.0f32; // allow more movement per step
                let center_x = 400.0f32;
                let center_y = 300.0f32;

                // Spatial grid for approximate repulsion
                use std::collections::HashMap;
                // Cell size must be larger than max node dimension to catch all overlaps
                let max_node_dim = widths.iter().chain(heights.iter()).cloned().fold(0.0f32, f32::max);
                let cell = (max_node_dim + 100.0).max(300.0f32);
                let mut bins: HashMap<(i32, i32), Vec<usize>> = HashMap::with_capacity(n * 2);
                for i in 0..n {
                    let gx = (xs[i] / cell).floor() as i32;
                    let gy = (ys[i] / cell).floor() as i32;
                    bins.entry((gx, gy)).or_default().push(i);
                }
                let neighbors = [
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (0, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ];
                for i in 0..n {
                    let gx = (xs[i] / cell).floor() as i32;
                    let gy = (ys[i] / cell).floor() as i32;
                    for (dxg, dyg) in neighbors {
                        if let Some(v) = bins.get(&(gx + dxg, gy + dyg)) {
                            for &j in v {
                                if j <= i {
                                    continue;
                                }
                                // Centers of nodes
                                let cx_i = xs[i] + widths[i] / 2.0;
                                let cy_i = ys[i] + heights[i] / 2.0;
                                let cx_j = xs[j] + widths[j] / 2.0;
                                let cy_j = ys[j] + heights[j] / 2.0;

                                let dx = cx_j - cx_i;
                                let dy = cy_j - cy_i;
                                let d2 = dx * dx + dy * dy + 0.01;
                                let inv = 1.0 / d2;
                                let fx_ij = repulsion * dx * inv;
                                let fy_ij = repulsion * dy * inv;
                                fx[i] -= fx_ij;
                                fy[i] -= fy_ij;
                                fx[j] += fx_ij;
                                fy[j] += fy_ij;
                            }
                        }
                    }
                }

                // Attraction along edges
                for edge in &edges {
                    let i = edge.source;
                    let j = edge.target;
                    if i >= n || j >= n {
                        continue;
                    }
                    let dx = xs[j] - xs[i];
                    let dy = ys[j] - ys[i];
                    let fx_e = attraction * dx;
                    let fy_e = attraction * dy;
                    fx[i] += fx_e;
                    fy[i] += fy_e;
                    fx[j] -= fx_e;
                    fy[j] -= fy_e;
                }

                // Gravity towards center
                for i in 0..n {
                    fx[i] += gravity * (center_x - xs[i]);
                    fy[i] += gravity * (center_y - ys[i]);
                }

                // Integrate and clamp small step
                for i in 0..n {
                    let mut dx = fx[i] * dt;
                    let mut dy = fy[i] * dt;
                    dx *= damping;
                    dy *= damping;
                    let disp2 = dx * dx + dy * dy;
                    if disp2 > max_disp * max_disp {
                        let s = max_disp / disp2.sqrt();
                        dx *= s;
                        dy *= s;
                    }
                    xs[i] += dx;
                    ys[i] += dy;
                }

                // Overlap resolution pass - push overlapping nodes apart directly
                let padding = 20.0f32; // Minimum gap between nodes
                for _ in 0..3 {
                    // Multiple iterations for better resolution
                    for i in 0..n {
                        for j in (i + 1)..n {
                            // Required separation
                            let sep_x = (widths[i] + widths[j]) / 2.0 + padding;
                            let sep_y = (heights[i] + heights[j]) / 2.0 + padding;

                            // Centers of nodes
                            let cx_i = xs[i] + widths[i] / 2.0;
                            let cy_i = ys[i] + heights[i] / 2.0;
                            let cx_j = xs[j] + widths[j] / 2.0;
                            let cy_j = ys[j] + heights[j] / 2.0;

                            let dx = cx_j - cx_i;
                            let dy = cy_j - cy_i;

                            let overlap_x = sep_x - dx.abs();
                            let overlap_y = sep_y - dy.abs();

                            if overlap_x > 0.0 && overlap_y > 0.0 {
                                // Push apart along the axis of least overlap
                                if overlap_x < overlap_y {
                                    // Push horizontally
                                    let push = overlap_x / 2.0 + 1.0;
                                    if dx >= 0.0 {
                                        xs[i] -= push;
                                        xs[j] += push;
                                    } else {
                                        xs[i] += push;
                                        xs[j] -= push;
                                    }
                                } else {
                                    // Push vertically
                                    let push = overlap_y / 2.0 + 1.0;
                                    if dy >= 0.0 {
                                        ys[i] -= push;
                                        ys[j] += push;
                                    } else {
                                        ys[i] += push;
                                        ys[j] -= push;
                                    }
                                }
                            }
                        }
                    }
                }

                // Write back
                for i in 0..n {
                    let nx = px(xs[i] as f32);
                    let ny = px(ys[i] as f32);
                    let ent = nodes_for_sim[i].clone();
                    cx.update_entity(&ent, move |node, _| {
                        node.x = nx;
                        node.y = ny;
                    });
                }
                // Bookkeep a tick so any observers can react and mark the graph dirty
                cx.update_entity(&graph_handle, |g: &mut Graph, _| {
                    g.sim_tick = g.sim_tick.wrapping_add(1);
                });
                cx.notify(graph_handle.entity_id());
            },
        )
        .absolute()
        .size_full();

        div()
            .size_full()
            // Background is transparent so parent can set the themed background
            .child(sim_canvas)
            // Clicking selects node under cursor; shift adds to selection; clicking empty space starts panning
            .on_mouse_down(
                gpui::MouseButton::Left,
                graph_cx.listener(|this, e: &gpui::MouseDownEvent, _w, cx| {
                    // Convert to container-local coordinates for hit testing
                    let cursor = point(
                        e.position.x - this.container_offset.x,
                        e.position.y - this.container_offset.y,
                    );
                    let mut hit_index: Option<usize> = None;
                    // Check each node using its actual width and height
                    for (i, n) in this.nodes.iter().enumerate() {
                        let (nx, ny, node_width, node_height) = cx.read_entity(n, |node, _| (node.x, node.y, node.width, node.height));
                        let left = this.pan.x + nx * this.zoom;
                        let top = this.pan.y + ny * this.zoom;
                        let scaled_width = px(node_width) * this.zoom;
                        let scaled_height = px(node_height) * this.zoom;
                        if cursor.x >= left
                            && cursor.x <= left + scaled_width
                            && cursor.y >= top
                            && cursor.y <= top + scaled_height
                        {
                            hit_index = Some(i);
                            break;
                        }
                    }

                    match hit_index {
                        Some(i) => {
                            let shift = e.modifiers.shift;
                            if !shift {
                                for n in &this.nodes {
                                    cx.update_entity(n, |node, _| node.selected = false);
                                }
                            }
                            let target = this.nodes[i].clone();
                            cx.update_entity(&target, |node, _| {
                                node.selected = true;
                            });
                        }
                        None => {
                            // No node hit - start panning
                            for n in &this.nodes {
                                cx.update_entity(n, |node, _| node.selected = false);
                            }
                            this.is_panning = true;
                            this.pan_start = this.pan;
                            this.pan_start_pos = cursor;
                        }
                    }
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                graph_cx.listener(|this, _e: &gpui::MouseUpEvent, _w, cx| {
                    this.is_panning = false;
                    cx.notify();
                }),
            )
            .on_scroll_wheel(graph_cx.listener({
                move |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let delta_px = event.delta.pixel_delta(px(16.0));
                    let dy = delta_px.y;

                    if dy != px(0.0) {
                        let factor = if dy > px(0.0) { 1.1 } else { 0.9 };
                        let old_zoom = this.zoom;
                        let new_zoom = (old_zoom * factor).clamp(0.25, 4.0);

                        // Zoom toward cursor position by adjusting pan
                        // Convert window position to container-local position
                        let s = point(
                            event.position.x - this.container_offset.x,
                            event.position.y - this.container_offset.y,
                        );
                        let world_x = (s.x - this.pan.x) / old_zoom;
                        let world_y = (s.y - this.pan.y) / old_zoom;
                        this.pan = point(s.x - world_x * new_zoom, s.y - world_y * new_zoom);

                        this.zoom = new_zoom;
                        for n in &this.nodes {
                            let pan = this.pan;
                            let zoom = this.zoom;
                            cx.update_entity(n, move |node, _| {
                                node.zoom = zoom;
                                node.pan = pan;
                            });
                        }
                        // ensure the graph re-renders so shared canvases reflect new zoom/pan
                        cx.notify();
                    }
                }
            }))
            // Middle mouse button for panning (alternative to left-click on empty space)
            .on_mouse_down(
                gpui::MouseButton::Middle,
                graph_cx.listener(|this, e: &gpui::MouseDownEvent, _w, cx| {
                    this.is_panning = true;
                    this.pan_start = this.pan;
                    this.pan_start_pos = point(
                        e.position.x - this.container_offset.x,
                        e.position.y - this.container_offset.y,
                    );
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Middle,
                graph_cx.listener(|this, _e: &gpui::MouseUpEvent, _w, cx| {
                    this.is_panning = false;
                    cx.notify();
                }),
            )
            .on_mouse_move(graph_cx.listener(|this, e: &gpui::MouseMoveEvent, _w, cx| {
                if this.is_panning {
                    let current_pos = point(
                        e.position.x - this.container_offset.x,
                        e.position.y - this.container_offset.y,
                    );
                    let delta = point(
                        current_pos.x - this.pan_start_pos.x,
                        current_pos.y - this.pan_start_pos.y,
                    );
                    this.pan = point(this.pan_start.x + delta.x, this.pan_start.y + delta.y);
                    
                    // Update all nodes with new pan
                    for n in &this.nodes {
                        let pan = this.pan;
                        cx.update_entity(n, move |node, _| {
                            node.pan = pan;
                        });
                    }
                    cx.notify();
                }
            }))
            .child(graph_canvas)
            .child(controls_panel)
            .child(
                // Play button for auto-layout simulation
                div()
                    .absolute()
                    .top(px(8.0))
                    .right(px(8.0))
                    .size(px(28.0))
                    .rounded_full()
                    .cursor_pointer()
                    .when(self.playing, |this| this.bg(rgb(0x4CAF50)))
                    .border(px(1.0))
                    .border_color(border_color)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(text_color)
                    .text_size(px(12.0))
                    .child(if self.playing { "⏸" } else { "▶" })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        graph_cx.listener({
                            move |this, _e: &gpui::MouseDownEvent, _w, cx| {
                                this.playing = !this.playing;
                                cx.notify();
                            }
                        }),
                    )
            )
    }
}
