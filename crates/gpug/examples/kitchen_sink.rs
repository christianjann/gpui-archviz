use gpug::*;
use gpui::{div, App, AppContext, Application, WindowOptions, *};

struct LayoutExample {
    graph: Entity<Graph>,
}

impl LayoutExample {
    fn new(cx: &mut Context<Self>) -> Self {
        let graph = cx.new(|cx| {
            let node_count = 25;
            let initial_k = 3;
            let initial_beta = 0.05;
            let nodes = generate_nodes(node_count);
            let edges = generate_watts_strogatz_graph(node_count, initial_k, initial_beta);
            let mut graph = Graph::new(cx, nodes, edges, initial_k, initial_beta);
            // Use Manhattan-style edge routing
            graph.edge_routing = EdgeRouting::Manhattan;
            graph
        });

        Self { graph }
    }
}

impl Render for LayoutExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .child(
                div()
                    .flex_1()
                    .bg(rgb(0xf0f0f0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("Left Panel (Empty)")
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.graph.clone())
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let mut window_opts = WindowOptions::default();
        window_opts.app_id = Some("GPUG Kitchen Sink".to_string());

        cx.open_window(window_opts, |_, cx| {
            cx.new(|cx| LayoutExample::new(cx))
        })
        .unwrap();
    });
}
