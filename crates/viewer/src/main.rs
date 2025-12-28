use gpui::*;
use gpui_component::{
    ActiveTheme,
    highlighter::Language,
    input::{Input, InputEvent, InputState, TabSize},
    resizable::{h_resizable, resizable_panel},
};
use gpui_component_assets::Assets;
use gpui_component_story::Open;
use gpug::{EdgeRouting, Graph, GpugEdge, GpugNode, NodeChild};
use tracing::error;
use std::collections::HashMap;

pub struct Example {
    input_state: Entity<InputState>,
    graph: Entity<Graph>,
    _subscriptions: Vec<Subscription>,
}

const EXAMPLE: &str = include_str!("../tests/model/vehicle.kdl");

/// Estimate node size based on name, type, and children (mirrors GpugNode::estimate_dimensions)
fn estimate_node_size(name: &str, node_type: &str, children: &[NodeChild]) -> (f32, f32) {
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
    let bus_y = 50.0f32;
    let ecu_y = 150.0f32;
    let start_x = 50.0f32;
    
    // First pass: collect all nodes with their estimated sizes
    struct NodeInfo {
        name: String,
        node_type: String,
        children: Vec<NodeChild>,
        estimated_width: f32,
        estimated_height: f32,
    }
    
    let mut bus_nodes: Vec<NodeInfo> = Vec::new();
    let mut ecu_nodes: Vec<NodeInfo> = Vec::new();
    
    for kdl_node in doc.nodes() {
        let name = kdl_node.name().to_string();
        
        // Check for type="ecu" or type="bus" in entries
        let node_type = kdl_node.entries().iter()
            .find(|e| e.name().map(|n| n.to_string().as_str() == "type").unwrap_or(false))
            .and_then(|e| e.value().as_string())
            .map(|s| s.to_string());
        
        if let Some(type_val) = node_type {
            node_name_to_index.insert(name.clone(), index);
            index += 1;
            
            // Extract children (partitions and swcs) for ECUs
            let children = if type_val == "ecu" {
                extract_node_children(kdl_node)
            } else {
                Vec::new()
            };
            
            // Estimate node size for layout
            let (estimated_width, estimated_height) = estimate_node_size(&name, &type_val, &children);
            
            let info = NodeInfo {
                name,
                node_type: type_val.clone(),
                children,
                estimated_width,
                estimated_height,
            };
            
            match type_val.as_str() {
                "bus" => bus_nodes.push(info),
                "ecu" => ecu_nodes.push(info),
                _ => {}
            }
        }
    }
    
    // Layout buses in a row with proper spacing
    let gap = 30.0f32;
    let mut bus_x = start_x;
    for info in &bus_nodes {
        nodes.push(GpugNode {
            id,
            name: info.name.clone(),
            node_type: info.node_type.clone(),
            children: info.children.clone(),
            x: px(bus_x),
            y: px(bus_y),
            drag_offset: None,
            zoom: 1.0,
            pan: point(px(0.0), px(0.0)),
            selected: false,
            container_offset: point(px(0.0), px(0.0)),
            width: info.estimated_width,
            height: info.estimated_height,
        });
        bus_x += info.estimated_width + gap;
        id += 1;
    }
    
    // Layout ECUs in a row below buses with proper spacing
    let mut ecu_x = start_x;
    for info in &ecu_nodes {
        nodes.push(GpugNode {
            id,
            name: info.name.clone(),
            node_type: info.node_type.clone(),
            children: info.children.clone(),
            x: px(ecu_x),
            y: px(ecu_y),
            drag_offset: None,
            zoom: 1.0,
            pan: point(px(0.0), px(0.0)),
            selected: false,
            container_offset: point(px(0.0), px(0.0)),
            width: info.estimated_width,
            height: info.estimated_height,
        });
        ecu_x += info.estimated_width + gap;
        id += 1;
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

/// Extract partition and swc children from a KDL node
fn extract_node_children(kdl_node: &kdl::KdlNode) -> Vec<NodeChild> {
    let mut children = Vec::new();
    
    if let Some(kdl_children) = kdl_node.children() {
        for child in kdl_children.nodes() {
            let node_name = child.name().to_string();
            
            // Check if this is a partition node (node name is "partition")
            if node_name == "partition" {
                // The partition name is the first positional argument (e.g., partition "GW_Routing")
                let partition_name = child.entries().iter()
                    .find(|e| e.name().is_none()) // positional argument has no name
                    .and_then(|e| e.value().as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unnamed".to_string());
                
                // Extract swcs inside the partition
                let swc_children = extract_swcs(child);
                
                children.push(NodeChild {
                    name: partition_name,
                    kind: "partition".to_string(),
                    children: swc_children,
                });
            }
        }
    }
    
    children
}

/// Extract SWC children from a partition node
fn extract_swcs(partition_node: &kdl::KdlNode) -> Vec<NodeChild> {
    let mut swcs = Vec::new();
    
    if let Some(kdl_children) = partition_node.children() {
        for child in kdl_children.nodes() {
            let node_name = child.name().to_string();
            
            // Check if this is an swc node (node name is "swc")
            if node_name == "swc" {
                // The swc name is the first positional argument (e.g., swc "CanRouter")
                let swc_name = child.entries().iter()
                    .find(|e| e.name().is_none()) // positional argument has no name
                    .and_then(|e| e.value().as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "unnamed".to_string());
                
                swcs.push(NodeChild {
                    name: swc_name,
                    kind: "swc".to_string(),
                    children: Vec::new(),
                });
            }
        }
    }
    
    swcs
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
                // Only update if we have valid nodes (KDL parsed successfully with content)
                if !nodes.is_empty() {
                    graph_for_sub.update(cx, |graph, cx| {
                        graph.update_model(nodes, edges, cx);
                    });
                } else {
                    error!("Document has errors, not updating graph!")
                }
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
