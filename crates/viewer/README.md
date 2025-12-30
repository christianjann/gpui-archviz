# Viewer

A demo application for visualizing vehicle E/E architecture diagrams using GPUI.

## Features

- Interactive graph visualization of architecture models
- Bidirectional highlighting between text editor and graph view
- Support for KDL-based model definitions
- Multiple layout algorithms (force-directed, basic hierarchical and ArchViz for architecture diagrams)
- Real-time updates as you edit the model

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Usage

1. Edit the KDL model in the left text editor
2. View the graph visualization in the right panel
3. Click graph nodes to select corresponding text ranges
4. Move cursor in text to highlight graph nodes

## Model Format

Uses KDL (KDL Document Language) for defining architecture models. See the main README for examples.

## Dependencies

- gpui: GPU-accelerated UI framework
- gpui-component: UI components
- graphview: Graph visualization library
- kdl: Parser for KDL files
