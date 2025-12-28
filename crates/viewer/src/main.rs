use gpui::*;
use gpui_component::{
    ActiveTheme,
    highlighter::Language,
    input::{Input, InputEvent, InputState, TabSize},
    resizable::{h_resizable, resizable_panel},
};
use gpui_component_assets::Assets;
use gpui_component_story::Open;
use gpug::{EdgeRouting, Graph, GpugEdge, GpugNode};
use std::collections::HashMap;

pub struct Example {
    input_state: Entity<InputState>,
    graph: Entity<Graph>,
    _subscriptions: Vec<Subscription>,
}

const EXAMPLE: &str = include_str!("../tests/model/vehicle.kdl");

/// Parse KDL content and extract nodes (ECUs and buses) with their connections
fn parse_kdl_model(content: &str) -> (Vec<GpugNode>, Vec<GpugEdge>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_name_to_index: HashMap<String, usize> = HashMap::new();
    
    // Parse the KDL document
    let doc = match kdl::KdlDocument::parse(content) {
        Ok(doc) => doc,
        Err(_) => return (nodes, edges),
    };
    
    // First pass: collect all nodes (ECUs and buses)
    let mut id: u64 = 1;
    let mut index: usize = 0;
    
    // Layout parameters for positioning nodes
    let bus_y = 100.0f32;
    let ecu_y = 300.0f32;
    let start_x = 100.0f32;
    let spacing = 200.0f32;
    
    let mut bus_count = 0usize;
    let mut ecu_count = 0usize;
    
    for kdl_node in doc.nodes() {
        let name = kdl_node.name().to_string();
        
        // Check for type="ecu" or type="bus" in entries
        let node_type = kdl_node.entries().iter()
            .find(|e| e.name().map(|n| n.to_string().as_str() == "type").unwrap_or(false))
            .and_then(|e| e.value().as_string())
            .map(|s| s.to_string());
        
        if let Some(type_val) = node_type {
            let (x, y) = match type_val.as_str() {
                "bus" => {
                    let pos = (start_x + bus_count as f32 * spacing, bus_y);
                    bus_count += 1;
                    pos
                }
                "ecu" => {
                    let pos = (start_x + ecu_count as f32 * spacing, ecu_y);
                    ecu_count += 1;
                    pos
                }
                _ => continue,
            };
            
            node_name_to_index.insert(name.clone(), index);
            
            nodes.push(GpugNode {
                id,
                name: name.clone(),
                x: px(x),
                y: px(y),
                drag_offset: None,
                zoom: 1.0,
                pan: point(px(0.0), px(0.0)),
                selected: false,
                container_offset: point(px(0.0), px(0.0)),
            });
            
            id += 1;
            index += 1;
        }
    }
    
    // Second pass: find interface connections from ECUs to buses
    for kdl_node in doc.nodes() {
        let ecu_name = kdl_node.name().to_string();
        
        // Check if this is an ECU
        let is_ecu = kdl_node.entries().iter()
            .find(|e| e.name().map(|n| n.to_string().as_str() == "type").unwrap_or(false))
            .and_then(|e| e.value().as_string())
            .map(|s| s == "ecu")
            .unwrap_or(false);
        
        if !is_ecu {
            continue;
        }
        
        // Get the ECU's index
        let Some(&ecu_index) = node_name_to_index.get(&ecu_name) else {
            continue;
        };
        
        // Look for interface children with bus= attribute
        if let Some(children) = kdl_node.children() {
            for child in children.nodes() {
                if child.name().to_string() == "interface" {
                    // Find the bus= attribute
                    if let Some(bus_name) = child.entries().iter()
                        .find(|e| e.name().map(|n| n.to_string().as_str() == "bus").unwrap_or(false))
                        .and_then(|e| e.value().as_string())
                    {
                        // Create edge from ECU to bus
                        if let Some(&bus_index) = node_name_to_index.get(bus_name) {
                            edges.push(GpugEdge::new(ecu_index, bus_index));
                        }
                    }
                }
            }
        }
    }
    
    (nodes, edges)
}

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
            graph
        });

        // Subscribe to input changes and update the graph
        let graph_for_sub = graph.clone();
        let _subscriptions = vec![cx.subscribe(&input_state, move |_this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                let content = input.read(cx).value();
                let (nodes, edges) = parse_kdl_model(&content);
                graph_for_sub.update(cx, |graph, cx| {
                    graph.update_model(nodes, edges, cx);
                });
            }
        })];

        Self {
            input_state,
            graph,
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

        gpui_component_story::create_new_window("KDL Model Editor", Example::view, cx);
    });
}
