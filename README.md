# GPUI Architecture Visualizer Demo

A demo application for experimenting with [GPUI](https://gpui.rs/) and layouting vehicle E/E architecture diagrams.

![Demo Screenshot](doc/screenshot.png)

The selection between the graph view and the code editor is automatically synchronized:
![Demo Screencast](doc/screencast.gif)

## About

This is an experimental project to explore:
- Building interactive graph visualizations with GPUI (Zed's GPU-accelerated UI framework)
- Layouting algorithms for architecture diagrams (Force-directed and Dagre/Sugiyama hierarchical layouts)
- Rendering nodes with nested children (ECUs with partitions and software components)
- Edge routing with Manhattan-style orthogonal paths

**Note:** This is a demo/prototype, not a production-ready tool.

## Why KDL?

[KDL](https://kdl.dev/) (the KDL Document Language) was chosen as the input format because:
- It's human-readable and easy to write
- Has an existing [tree-sitter grammar](https://github.com/tree-sitter-grammars/tree-sitter-kdl) for syntax highlighting
- Well-suited for hierarchical data like architecture models

**Future Direction:** In the future, this may be based on [SysML v2](https://www.omgsysml.org/SysML-2.htm) or another architecture DSL. However, creating a proper SysML tree-sitter grammar or parser is too much effort just for experimenting with GPUI. KDL serves as a lightweight stand-in for now.

## Technology Stack

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[GPUI](https://gpui.rs/)** - GPU-accelerated UI framework from Zed
- **[gpui-component](https://github.com/longbridge/gpui-component)** - UI component library for GPUI
- **[dagre-rs](https://crates.io/crates/dagre-rs)** - Hierarchical graph layout (Sugiyama method)
- **[petgraph](https://crates.io/crates/petgraph)** - Graph data structures
- **[KDL](https://kdl.dev/)** - Document language for model files

## Features

- 📊 **Graph Visualization** - Interactive node-based diagrams
- 🔄 **Multiple Layouts** - Force-directed simulation or Dagre hierarchical layout
- 🖱️ **Interactive** - Pan, zoom, drag nodes, click to select
- 🎨 **Edge Highlighting** - Orange for outgoing edges, blue for incoming edges
- 📦 **Nested Nodes** - ECUs contain partitions which contain software components
- ↔️ **Manhattan Routing** - Clean orthogonal edge paths
- 🔄 **Bidirectional Highlighting** - Text edits highlight corresponding graph nodes; graph node clicks select and center text ranges; cursor movement in text highlights graph nodes

## Building

```bash
# Clone the repository
git clone https://github.com/christianjann/gpui-archviz
cd gpui-archviz

# Build and run
cargo run
```

## Usage

1. Edit the KDL model in the left panel to see graph updates
2. Click graph nodes to select and center the corresponding text range
3. Move the cursor in the text editor to highlight the associated graph node
4. Use the controls:
   - **Zoom**: `+`/`-` buttons or mouse wheel
   - **Pan**: Middle mouse button drag or scroll
   - **Fit**: Click "Fit" to fit all nodes in view
   - **Layout**: Toggle between "Force" and "Dagre" layout modes
   - **Play/Refresh**: In Force mode, toggles simulation; in Dagre mode, re-applies layout

## Model Format (KDL)

```kdl
// Define a CAN bus
CAN type="bus"

// Define an ECU with partitions and software components
BodyController type="ecu" {
    partition name="Safety" {
        swc name="AirbagController"
        swc name="ABSController"
    }
    interface bus="CAN"
}
```
## Known issues
- When maximizing the window the cursor stays in resize mode outside the original window area until first manual resize of the window
- Sometimes the draping cursor is stuck at dragging, what helps is a right or middle click of the mouse

## Next steps
- Implement better auto-layout

## License

MIT
