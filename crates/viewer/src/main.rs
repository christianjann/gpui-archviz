use gpui::*;
use gpui_component::{
    ActiveTheme,
    highlighter::Language,
    input::{Input, InputEvent, InputState, TabSize},
    resizable::{h_resizable, resizable_panel},
};
use gpui_component_assets::Assets;
use gpui_component_story::Open;
use graphview::{EdgeRouting, Graph, NodeSelected};
use tracing::{info, error};
use lsp_types::Position;

mod kdl;
use kdl::parse_kdl_model;

pub struct Example {
    input_state: Entity<InputState>,
    graph: Entity<Graph>,
    last_cursor_pos: Option<Position>,
    _subscriptions: Vec<Subscription>,
}

/// Convert line and character position to byte offset in the text
fn line_char_to_offset(text: &str, line: usize, character: usize) -> usize {
    let mut current_line = 0;
    let mut current_char = 0;
    for (i, ch) in text.char_indices() {
        if current_line == line && current_char == character {
            return i;
        }
        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }
    text.len() // if beyond the text
}

const EXAMPLE: &str = include_str!("../tests/model/vehicle.kdl");

impl Example {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(Language::Kdl)
                .line_number(true)
                .tab_size(TabSize {
                    tab_size: 2,
                    ..Default::default()
                })
                .searchable(true)
                .placeholder("Enter your KDL diagram here...")
                .default_value(EXAMPLE)
        });

        // Parse the KDL content and create nodes/edges from ECUs and buses
        let (nodes, edges) = parse_kdl_model(EXAMPLE);
        let node_count = nodes.len();

        let graph = cx.new(|_cx| {
            let mut graph = Graph::new(_cx, nodes, edges, 3, 0.05);
            // Use Manhattan-style edge routing
            graph.edge_routing = EdgeRouting::Manhattan;
            // Trigger layout since we're providing positioned nodes
            graph.needs_layout = node_count == 0;
            // Fit to window on initial render
            graph.needs_fit_to_content = true;
            graph
        });

        // Subscribe to input changes and update the graph
        let graph_for_sub = graph.clone();
        let input_state_for_graph = input_state.clone();
        let _subscriptions =
            vec![
                cx.subscribe(&input_state, move |_this, input, event: &InputEvent, cx| {
                    if let InputEvent::Change = event {
                        let content = input.read(cx).value();
                        let (nodes, edges) = parse_kdl_model(&content);
                        // Only update if we have valid nodes (KDL parsed successfully with content)
                        if !nodes.is_empty() {
                            graph_for_sub.update(cx, |graph, cx| {
                                graph.update_model(nodes, edges, cx);
                            });
                        } else {
                            error!("Document has errors, not updating graph!")
                        }
                    }
                }),
                cx.observe(&input_state, move |this, input, cx| {
                    let pos = input.read(cx).cursor_position();
                    if Some(pos) != this.last_cursor_pos {
                        this.last_cursor_pos = Some(pos);
                        let content = input.read(cx).value();
                        let cursor = line_char_to_offset(&content, pos.line as usize, pos.character as usize);
                        info!("Cursor moved to byte position: {}", cursor);
                        this.graph.update(cx, |graph, cx| {
                            // Deselect all
                            for n in &graph.nodes {
                                cx.update_entity(n, |node, _| node.selected = false);
                            }
                            // Select the node at cursor
                            for node_entity in &graph.nodes {
                                let span = cx.read_entity(node_entity, |n, _| n.span);
                                if let Some((s, e)) = span {
                                    if s <= cursor && cursor <= e {
                                        let node_name = cx.read_entity(node_entity, |n, _| n.name.clone());
                                        cx.update_entity(node_entity, |node, _| node.selected = true);
                                        info!("Selected node: {}", node_name);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                }),
                cx.subscribe(&graph, move |_this, _graph, event: &NodeSelected, cx| {
                    info!("Node selected in graph: id={}, span={:?}", event.node_id, event.span);
                    // Set text selection to event.span
                    if let Some((start, end)) = event.span {
                        info!("Setting text selection to {}..{}", start, end);
                        input_state_for_graph.update(cx, |input, cx| {
                            input.set_selection_range(start, end, cx);
                        });
                    }
                }),
            ];

        Self {
            input_state,
            graph,
            last_cursor_pos: None,
            _subscriptions,
        }
    }

    fn on_action_open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: true,
            multiple: false,
            prompt: Some("Select a KDL file".into()),
        });

        let input_state = self.input_state.clone();
        cx.spawn_in(window, async move |_, window| {
            let path = path.await.ok()?.ok()??.iter().next()?.clone();

            let content = std::fs::read_to_string(&path).ok()?;

            window
                .update(|window, cx| {
                    _ = input_state.update(cx, |this, cx| {
                        this.set_value(content, window, cx);
                    });
                })
                .ok();

            Some(())
        })
        .detach();
    }

    fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Render for Example {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("editor")
            .size_full()
            .on_action(cx.listener(Self::on_action_open))
            .child(
                h_resizable("container")
                    .child(
                        resizable_panel().child(
                            div()
                                .id("source")
                                .size_full()
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_size(cx.theme().mono_font_size)
                                .child(
                                    Input::new(&self.input_state)
                                        .h_full()
                                        .p_0()
                                        .border_0()
                                        .focus_bordered(false),
                                ),
                        ),
                    )
                    .child(
                        resizable_panel().child(
                            div()
                                .id("graph-preview")
                                .relative()
                                .w_full()
                                .h_full()
                                .overflow_hidden()
                                .bg(cx.theme().background)
                                .child(self.graph.clone()),
                        ),
                    ),
            )
    }
}

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component_story::init(cx);
        cx.activate(true);

        gpui_component_story::create_new_window_with_size(
            "GPUI Architecture Visualizer",
            Some(size(px(1200.0), px(800.0))),
            Example::view,
            cx,
        );
    });
}
