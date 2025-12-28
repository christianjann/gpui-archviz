use gpui::div;
use gpui::*;

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
        
        // Left port (incoming)
        let left_port = div()
            .absolute()
            .left(px(-port_size / 2.0 * self.zoom))
            .top(px((base_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0x4488ff))
            .border(px(1.0))
            .border_color(rgb(0x333333))
            .rounded(px(2.0 * self.zoom));
        
        // Right port (outgoing)
        let right_port = div()
            .absolute()
            .right(px(-port_size / 2.0 * self.zoom))
            .top(px((base_height - port_size) / 2.0 * self.zoom))
            .size(px(port_size * self.zoom))
            .bg(rgb(0xff8844))
            .border(px(1.0))
            .border_color(rgb(0x333333))
            .rounded(px(2.0 * self.zoom));
        
        let node = div()
            .relative()
            .min_w(px(base_width * self.zoom))
            .h(px(base_height * self.zoom))
            .px(px(12.0 * self.zoom)) // More padding for ports
            .bg(rgb(0xffffff))
            .border(px(2.0))
            .border_color(if self.selected { rgb(0x1E90FF) } else { rgb(0x333333) })
            .rounded(px(4.0 * self.zoom))
            .shadow_sm()
            .cursor_move()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0x000000))
            .text_size(px(12.0 * self.zoom))
            .child(left_port)
            .child(self.name.clone())
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
            .child(node)
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
