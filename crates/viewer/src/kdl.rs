//! KDL model parsing utilities for extracting graph nodes and edges

use graphview::{GraphEdge, GraphNode, NodeChild};
use gpui::{point, px};
use std::collections::HashMap;

/// Parse KDL content and extract nodes (ECUs and buses) with their connections
pub fn parse_kdl_model(content: &str) -> (Vec<GraphNode>, Vec<GraphEdge>) {
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
            let (estimated_width, estimated_height) = GraphNode::estimate_node_size(&name, &type_val, &children);
            
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
        nodes.push(GraphNode {
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
        nodes.push(GraphNode {
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
                            edges.push(GraphEdge::new(ecu_index, bus_index));
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
