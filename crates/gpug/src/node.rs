use gpui::div;
use gpui::*;
use gpui_component::ActiveTheme;

// Simple draggable node with label
pub struct GpugNode {
    pub id: u64,
    pub name: String,
    pub x: Pixels,
    pub y: Pixels,
    // Offset from the node's origin to the cursor at drag start
    pub drag_offset: Option<Point<Pixels>>,
    pub zoom: f32,
    pub pan: Point<Pixels>,
    pub selected: bool,
    // Container offset in window coordinates (set by Graph during render)
    pub container_offset: Point<Pixels>,
}

impl Render for GpugNode {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base_width = 80.0;
        let base_height = 32.0;
        let port_size = base_height / 10.0 * 3.0; // Port is about 1/10th of node, but visible
        
        // Get theme colors
        let text_color = cx.theme().foreground;
        let border_color = cx.theme().border;
        let bg_color = cx.theme().secondary;
        let selected_border = cx.theme().ring;
        
        // Left port (incoming) - positioned as sibling after node so it renders on top
        let left_port = div()
            .absolute()
            .left(px(-port_size / 2.0 * self.zoom))
            .top(px((base_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0x4488ff))
            .border(px(1.0))
            .border_color(border_color)
            .rounded(px(2.0 * self.zoom));
        
        // Right port (outgoing) - positioned as sibling after node so it renders on top
        let right_port = div()
            .absolute()
            .left(px((base_width - port_size / 2.0) * self.zoom))
            .top(px((base_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0xff8844))
            .border(px(1.0))
            .border_color(border_color)
            .rounded(px(2.0 * self.zoom));
        
        // Node body (rendered first, so ports appear on top)
        let node_body = div()
            .absolute()
            .min_w(px(base_width * self.zoom))
            .h(px(base_height * self.zoom))
            .px(px(12.0 * self.zoom))
            .bg(bg_color)
            .border(px(2.0))
            .border_color(if self.selected { selected_border } else { border_color })
            .rounded(px(4.0 * self.zoom))
            .shadow_sm()
            .flex()
            .items_center()
            .justify_center()
            .text_color(text_color)
            .text_size(px(12.0 * self.zoom))
            .child(self.name.clone());

        // Container for node + ports with interaction handlers
        let node_container = div()
            .relative()
            .min_w(px(base_width * self.zoom))
            .h(px(base_height * self.zoom))
            .cursor_move()
            // Node body first, then ports on top
            .child(node_body)
            .child(left_port)
            .child(right_port)
            .id(("node", self.id as usize))
            // Start a drag with this node's id as payload; lets listeners filter events
            .on_drag(self.id, |_id: &u64, _offset, _window, cx| {
                cx.new(|_| DragPreview)
            })
            // Update position while dragging only if this node is the dragged one
            .on_drag_move::<u64>(
                cx.listener(|this, event: &DragMoveEvent<u64>, _window, cx| {
                    if *event.drag(cx) != this.id {
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
                        // position changes trigger re-render
                    }
                }),
            )
            .on_drop(cx.listener(|this, dragged_id: &u64, _window, _cx| {
                if *dragged_id == this.id {
                    this.drag_offset = None;
                }
            }));

        // Position the node with absolute positioning
        div()
            .absolute()
            .left(self.pan.x + self.x * self.zoom)
            .top(self.pan.y + self.y * self.zoom)
            .child(node_container)
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
