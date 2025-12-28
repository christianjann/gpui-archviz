# Graph

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

## Dependencies

- [GPUI](https://gpui.rs/) - GPU-accelerated UI framework
- [dagre-rs](https://crates.io/crates/dagre-rs) - Hierarchical graph layout
- [petgraph](https://crates.io/crates/petgraph) - Graph data structures
