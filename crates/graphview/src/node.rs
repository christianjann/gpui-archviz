use gpui::div;
use gpui::*;
use gpui_component::ActiveTheme;

/// Child element inside a node (partition or swc)
#[derive(Clone, Debug)]
pub struct NodeChild {
    pub name: String,
    pub kind: String, // "partition" or "swc"
    pub children: Vec<NodeChild>,
}

// Simple draggable node with label
pub struct GraphNode {
    pub id: u64,
    pub name: String,
    pub node_type: String, // "ecu", "bus", "interface", etc.
    pub x: Pixels,
    pub y: Pixels,
    // Offset from the node's origin to the cursor at drag start
    pub drag_offset: Option<Point<Pixels>>,
    pub zoom: f32,
    pub pan: Point<Pixels>,
    pub selected: bool,
    // Container offset in window coordinates (set by Graph during render)
    pub container_offset: Point<Pixels>,
    // Actual node width (calculated based on text content)
    pub width: f32,
    // Actual node height (calculated based on children)
    pub height: f32,
    // Child elements (partitions, swcs)
    pub children: Vec<NodeChild>,
}

impl GraphNode {
    /// Estimate node size based on name, type, and children (static version for pre-creation estimation)
    pub fn estimate_node_size(name: &str, node_type: &str, children: &[NodeChild]) -> (f32, f32) {
        let base_width = 120.0f32;
        let header_height = 28.0f32;
        let char_width = 7.2f32;
        let padding = 24.0f32;
        
        // Width from name and type
        let name_width = name.len() as f32 * char_width + padding;
        let type_width = node_type.len() as f32 * 6.0 + 40.0;
        let mut content_width = name_width.max(type_width).max(base_width);
        
        if children.is_empty() {
            return (content_width, header_height);
        }
        
        // Estimate height: header + partitions stacked vertically
        let mut total_child_height = 0.0f32;
        let mut max_partition_width = 0.0f32;
        
        for child in children {
            let swc_count = child.children.len().max(1);
            let partition_height = 40.0 + (swc_count as f32 * 45.0);
            total_child_height += partition_height + 8.0;
            
            let partition_name_width = child.name.len() as f32 * 6.0 + 50.0;
            let max_swc_width = child.children.iter()
                .map(|s| s.name.len() as f32 * 6.0 + 50.0)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(60.0);
            let partition_width = partition_name_width.max(max_swc_width * 2.0 + 20.0);
            max_partition_width = max_partition_width.max(partition_width);
        }
        
        content_width = content_width.max(max_partition_width + 16.0);
        let node_height = header_height + total_child_height + 12.0;
        
        (content_width, node_height)
    }

    /// Estimate node dimensions for hit testing (conservative/larger estimate)
    pub fn estimate_dimensions(&self) -> (f32, f32) {
        Self::estimate_node_size(&self.name, &self.node_type, &self.children)
    }
    
    /// Render a child element (partition or swc) recursively
    fn render_child(child: &NodeChild, zoom: f32, text_color: Hsla, border_color: Hsla) -> Div {
        let char_width = 6.0f32;
        let padding = 16.0f32;
        let child_width = (child.name.len() as f32 * char_width + padding).max(60.0);
        
        // Different background colors for partition vs swc
        let (bg, label_color) = match child.kind.as_str() {
            "partition" => (rgb(0x2a4a6a), rgb(0x88aacc)), // Blue-ish for partitions
            "swc" => (rgb(0x3a5a3a), rgb(0x88cc88)),       // Green-ish for swc
            _ => (rgb(0x4a4a4a), rgb(0xaaaaaa)),
        };
        
        let mut container = div()
            .m(px(2.0 * zoom))
            .p(px(4.0 * zoom))
            .min_w(px(child_width * zoom))
            .bg(bg)
            .border(px(1.0))
            .border_color(border_color)
            .rounded(px(3.0 * zoom))
            .flex()
            .flex_col()
            .gap(px(2.0 * zoom))
            .child(
                div()
                    .text_size(px(9.0 * zoom))
                    .text_color(label_color)
                    .child(format!("«{}»", child.kind))
            )
            .child(
                div()
                    .text_size(px(10.0 * zoom))
                    .text_color(text_color)
                    .child(child.name.clone())
            );
        
        // Add nested children (swcs inside partitions)
        if !child.children.is_empty() {
            let children_container = div()
                .flex()
                .flex_wrap()
                .gap(px(2.0 * zoom))
                .children(
                    child.children.iter().map(|c| Self::render_child(c, zoom, text_color, border_color))
                );
            container = container.child(children_container);
        }
        
        container
    }
}

impl Render for GraphNode {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header_height = 28.0f32;
        let port_size = 10.0f32;
        
        // Get theme colors
        let text_color = cx.theme().foreground;
        let border_color = cx.theme().border;
        let bg_color = cx.theme().secondary;
        let selected_border = cx.theme().ring;
        
        // Type-specific colors
        let (type_bg, type_label_color) = match self.node_type.as_str() {
            "ecu" => (rgb(0x4a3a6a), rgb(0xcc88ff)),  // Purple for ECU
            "bus" => (rgb(0x6a5a3a), rgb(0xffcc88)),  // Orange for bus
            _ => (rgb(0x4a4a4a), rgb(0xaaaaaa)),
        };
        
        // Calculate node dimensions - use estimate_dimensions for consistency
        let (node_width, node_height) = self.estimate_dimensions();
        let has_children = !self.children.is_empty();
        
        // Update stored dimensions for edge routing and hit testing
        self.width = node_width;
        self.height = node_height;
        
        // Header with type label and name
        let header = div()
            .w_full()
            .px(px(8.0 * self.zoom))
            .py(px(4.0 * self.zoom))
            .flex()
            .items_center()
            .gap(px(8.0 * self.zoom))
            .child(
                // Type badge
                div()
                    .px(px(4.0 * self.zoom))
                    .py(px(1.0 * self.zoom))
                    .bg(type_bg)
                    .rounded(px(2.0 * self.zoom))
                    .text_size(px(9.0 * self.zoom))
                    .text_color(type_label_color)
                    .child(format!("«{}»", self.node_type))
            )
            .child(
                // Name
                div()
                    .text_size(px(11.0 * self.zoom))
                    .text_color(text_color)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.name.clone())
            );
        
        // Children container (partitions and swcs) - stack vertically
        let children_container = if has_children {
            Some(
                div()
                    .w_full()
                    .px(px(4.0 * self.zoom))
                    .pb(px(4.0 * self.zoom))
                    .flex()
                    .flex_col() // Stack partitions vertically
                    .gap(px(4.0 * self.zoom))
                    .children(
                        self.children.iter().map(|c| Self::render_child(c, self.zoom, text_color, border_color))
                    )
            )
        } else {
            None
        };
        
        // Left port (incoming)
        let left_port = div()
            .absolute()
            .left(px(-port_size / 2.0 * self.zoom))
            .top(px((header_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0x4488ff))
            .border(px(1.0))
            .border_color(border_color)
            .rounded(px(2.0 * self.zoom));
        
        // Right port (outgoing)
        let right_port = div()
            .absolute()
            .right(px(-port_size / 2.0 * self.zoom))
            .top(px((header_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0xff8844))
            .border(px(1.0))
            .border_color(border_color)
            .rounded(px(2.0 * self.zoom));
        
        // Node body - use fixed width for consistent edge routing
        let mut node_body = div()
            .id(("node", self.id as usize))
            .w(px(node_width * self.zoom))
            .bg(bg_color)
            .border(px(2.0))
            .border_color(if self.selected { selected_border } else { border_color })
            .rounded(px(4.0 * self.zoom))
            .shadow_sm()
            .flex()
            .flex_col()
            .cursor_move()
            .child(header)
            // Start a drag with this node's id as payload
            .on_drag(self.id, |_id: &u64, _offset, _window, cx| {
                cx.new(|_| DragPreview)
            })
            // Update position while dragging
            .on_drag_move::<u64>(
                cx.listener(|this, event: &DragMoveEvent<u64>, _window, _cx| {
                    if *event.drag(_cx) != this.id {
                        return;
                    }
                    
                    // Record the initial cursor offset inside the node on first move
                    if this.drag_offset.is_none() {
                        let offset = point(
                            (event.event.position.x - event.bounds.left()) / this.zoom,
                            (event.event.position.y - event.bounds.top()) / this.zoom,
                        );
                        this.drag_offset = Some(offset);
                    }

                    if let Some(offset) = this.drag_offset {
                        // Subtract container_offset to convert window coords to container coords
                        let new_origin = point(
                            (event.event.position.x - this.container_offset.x - this.pan.x) / this.zoom - offset.x,
                            (event.event.position.y - this.container_offset.y - this.pan.y) / this.zoom - offset.y,
                        );
                        this.x = new_origin.x;
                        this.y = new_origin.y;
                    }
                }),
            )
            .on_drop(cx.listener(|this, dragged_id: &u64, _window, _cx| {
                if *dragged_id == this.id {
                    this.drag_offset = None;
                }
            }));
        
        if let Some(children) = children_container {
            node_body = node_body.child(children);
        }

        // Wrapper to position ports relative to node_body
        let node_wrapper = div()
            .relative()
            .child(node_body)
            .child(left_port)
            .child(right_port);

        // Position the node with absolute positioning
        div()
            .absolute()
            .left(self.pan.x + self.x * self.zoom)
            .top(self.pan.y + self.y * self.zoom)
            .child(node_wrapper)
    }
}

// Minimal drag preview view to satisfy on_drag constructor
struct DragPreview;
impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Invisible 1x1 element as drag ghost
        div().size(px(1.0)).bg(rgb(0xffffff)).opacity(0.0)
    }
}
