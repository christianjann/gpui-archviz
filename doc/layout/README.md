# Layout Algorithms Documentation

This directory contains detailed documentation for various graph layout algorithms used in the GPUI Architecture Visualizer.

## Available Layout Algorithms

### [Custom ArciVis Layout](custom.md)
The primary layout algorithm developed specifically for this project, optimized for vehicle E/E architecture diagrams.

- **Type**: Force-directed with orthogonal edge routing
- **Best For**: Architecture diagrams, technical schematics
- **Features**: Obstacle avoidance, port-aware routing, dynamic extensions
- **Implementation**: Custom Rust implementation

### [Force-Directed Layout](force.md)
Classical physics-based layout algorithm that treats nodes as particles and edges as springs.

- **Type**: Physics simulation
- **Best For**: General graphs, organic layouts
- **Features**: Natural-looking results, handles arbitrary graphs
- **Implementation**: Custom Rust implementation

### [Dagre Layout](dagre.md)
JavaScript library implementing the Sugiyama hierarchical layout algorithm.

- **Type**: Hierarchical (layered)
- **Best For**: Flowcharts, organizational charts, dependency graphs
- **Features**: Layer assignment, crossing reduction, orthogonal routing
- **Implementation**: Rust port of dagre-js

### [ELK Layout](elk.md)
Eclipse Layout Kernel - a comprehensive layout framework with multiple algorithms.

- **Type**: Modular framework (primarily hierarchical)
- **Best For**: Complex diagrams, custom layout requirements
- **Features**: Configurable pipeline, multiple algorithms, extensible
- **Implementation**: Not currently integrated (reference only)

## Comparison

| Algorithm | Type | Routing | Best Use Case | Performance |
|-----------|------|---------|---------------|-------------|
| Custom ArciVis | Force-directed + Orthogonal | Manhattan with obstacles | Architecture diagrams | Medium |
| Force-Directed | Physics simulation | Straight lines | General graphs | Medium |
| Dagre | Hierarchical | Orthogonal | Flowcharts | Fast |
| ELK | Modular | Configurable | Complex layouts | Variable |

## Implementation Status

- ✅ **Custom ArciVis**: Fully implemented and integrated
- ✅ **Force-Directed**: Fully implemented and integrated
- ✅ **Dagre**: Fully implemented and integrated
- ❌ **ELK**: Documented for reference, not implemented

## Navigation

- [Main Project README](../../README.md) - Project overview and features
- [Layout Crate Documentation](https://github.com/christianjann/arcivis-layout) - API reference
- [GraphView Crate Documentation](../../crates/graphview/README.md) - Integration guide