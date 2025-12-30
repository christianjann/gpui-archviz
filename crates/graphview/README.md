# GraphView

A GPUI-based graph visualization library for interactive node-edge diagrams.

## Attribution

This crate is based on [gpug](https://github.com/jerlendds/gpug) by [@jerlendds](https://github.com/jerlendds).

Based on commit [`ed5af44`](https://github.com/jerlendds/gpug/commit/ed5af446a091e3c0e19bebbcc4696f13e0b24cf0) (2025-10-16).

Thank you to jerlendds for the original implementation!

**License Status:** The license for the original gpug repository is currently unknown. If the upstream license is clarified, this will be updated accordingly or the code will be removed.

## Features

- **Node Rendering** - Customizable nodes with headers, nested children, and ports
- **Edge Routing** - Straight lines or Manhattan-style orthogonal routing
- **Layout Algorithms**:
  - Force-directed simulation with collision avoidance
  - Dagre hierarchical layout (Sugiyama method)
  - ArchViz layout optimized for architecture diagrams with obstacle avoidance
- **Interactions** - Pan, zoom, drag nodes, click to select
- **Edge Highlighting** - Visual feedback for selected node connections

## Usage

```rust
use graph::{Graph, GraphNode, GraphEdge, EdgeRouting, LayoutMode};

// Create nodes and edges
let nodes = vec![
    GraphNode { id: 1, name: "Node A".into(), /* ... */ },
    GraphNode { id: 2, name: "Node B".into(), /* ... */ },
];
let edges = vec![
    GraphEdge::new(0, 1),
];

// Create the graph
let graph = cx.new(|cx| Graph::new(cx, nodes, edges, 4, 0.3));
```

## Coordinate System Details

**Layout Crate Coordinate System:**
- Uses `f64` abstract units (no physical meaning)
- Origin at (0,0) with positive X right, positive Y down
- Grid-aligned positioning (multiples of 5.0 units)
- Layout results are centered and scaled to fit canvas

**GPUI/Graphview Coordinate System:**
- Uses `Pixels` (physical screen units)
- Supports zoom and pan transformations
- Origin can be anywhere (affected by pan)
- Coordinates: `screen_pos = pan + world_pos * zoom`

**Integration Challenges:**
1. **Unit Conversion**: Layout crate uses abstract units, GPUI uses pixels
2. **Zoom Handling**: Layout results need scaling for different zoom levels
3. **Origin Offset**: Layout crate centers results, GPUI has arbitrary origins
4. **Real-time Updates**: Layout is batch operation, GPUI needs incremental updates

**Recommended Integration:**
```rust
// Convert GPUI pixels to layout units
let layout_units_per_pixel = 1.0 / zoom_factor;
let layout_nodes = gpui_nodes.iter().map(|node| {
    LayoutNode {
        position: Position {
            x: (node.x - pan.x) / zoom_factor / layout_units_per_pixel,
            y: (node.y - pan.y) / zoom_factor / layout_units_per_pixel,
        },
        size: Size {
            width: node.width / layout_units_per_pixel,
            height: node.height / layout_units_per_pixel,
        },
        // ... other fields
    }
});

// Run layout
layout.layout_in_place(&mut layout_nodes, &mut layout_edges)?;

// Convert back to GPUI coordinates
for (gpui_node, layout_node) in gpui_nodes.iter_mut().zip(layout_nodes) {
    gpui_node.x = pan.x + layout_node.position.x * layout_units_per_pixel * zoom_factor;
    gpui_node.y = pan.y + layout_node.position.y * layout_units_per_pixel * zoom_factor;
}
```

## Dependencies

- [GPUI](https://gpui.rs/) - GPU-accelerated UI framework
- [archviz-layout](../layout/) - Custom orthogonal graph layout with obstacle avoidance
- [dagre-rs](https://crates.io/crates/dagre-rs) - Hierarchical graph layout
- [petgraph](https://crates.io/crates/petgraph) - Graph data structures
