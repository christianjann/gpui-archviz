# Custom ArchViz Layout Algorithm

This document outlines the custom graph layout algorithm implemented in the [ArchViz Layout crate](../crates/layout/README.md). For API documentation and usage examples, see the [main crate documentation](../crates/layout/README.md).

## Current Implementation

The algorithm is fully implemented and consists of four phases that produce clean, readable graph layouts for architectural visualization.

### Phase 1: Initial Placement
- **Simple Grid Layout**: Nodes are placed in a grid pattern based on connectivity
- **Size-Aware Positioning**: Larger nodes are given priority in initial placement
- **Basic Spacing**: Initial minimum spacing prevents immediate overlaps

### Phase 2: Force-Directed Refinement
- **Repulsion Forces**: Nodes repel each other based on their bounding boxes
- **Attraction Forces**: Connected nodes attract along their connecting edges
- **Iterative Optimization**: 200 iterations (configurable) with cooling schedule
- **Configurable Parameters**:
  - `repulsion_strength`: Default 10,000.0
  - `attraction_strength`: Default 0.1
  - `min_spacing`: Default 50.0 (configurable per test set)

### Phase 3: Edge Routing
- **Port Selection**: Intelligent port assignment based on connection direction
- **Grid-Based Pathfinding**: BFS algorithm on discrete 5.0-unit grid
- **Orthogonal Routing**: All edges use horizontal/vertical segments only
- **Obstacle Avoidance**: Routes avoid node interiors and existing edges
- **Extension Points**: 25.0-unit orthogonal extensions from ports before routing

### Phase 4: Finalization
- **Canvas Calculation**: Determines bounding box with padding
- **Centering**: Translates layout to center on canvas (top-left at 0,0)
- **Path Optimization**: Ensures all segments are properly orthogonal

## Key Features Implemented

- **Variable Node Sizes**: Fully supported with bounding box collision detection
- **Port System**: Up to 8 ports per node with input/output types
- **Orthogonal Edge Routing**: Manhattan-style routing with 90-degree bends
- **Obstacle Avoidance**: Grid-based pathfinding avoids node bodies
- **Force-Directed Layout**: Physics-based positioning with convergence
- **Configurable Parameters**: Adjustable forces, spacing, and iteration counts
- **Two API Variants**: Ownership-based and in-place modification APIs

## Test Sets

The implementation includes 5 comprehensive test sets demonstrating different scenarios:

1. **Test Set 1**: Basic network topology with ports completely outside nodes
2. **Test Set 2**: Complex structure with spaced edges option
3. **Test Set 3**: Automotive systems with ports centered on node edges (half in/half out)
4. **Test Set 4**: Complex automotive system with dense connectivity
5. **Test Set 5**: ECU network with separate bus and Ethernet clusters

## Performance Characteristics

- **Target Scale**: Optimized for 50-200 nodes typical in architecture diagrams
- **Algorithm Complexity**: O(n²) for force-directed phase, O(e × grid_size) for routing
- **Grid Resolution**: 5.0-unit cells for pathfinding precision
- **Iteration Count**: Fixed 200 iterations for predictable performance
- **Memory Usage**: Efficient data structures with minimal allocations

## Configuration Options

```rust
CustomLayout {
    iterations: 200,                    // Force-directed iterations
    repulsion_strength: 10000.0,        // Node repulsion force
    attraction_strength: 0.1,           // Edge attraction force
    min_spacing: 50.0,                  // Minimum node spacing
    allow_diagonals: false,             // Orthogonal routing only
    spaced_edges: false,                // Allow edge overlaps
}
```

## Integration

The layout crate integrates with the broader GPUI architecture visualization system:

- **Node Types**: Compatible with GPUI entity system
- **Port System**: Supports up to 8 ports per node with directional preferences
- **Edge Routing**: Produces waypoint arrays for smooth curve rendering
- **Canvas Output**: Provides final dimensions for proper centering

## Future Enhancements

- **Hierarchical Layout**: Support for nested node groups
- **Incremental Updates**: Partial layout updates for dynamic graphs
- **GPU Acceleration**: Leverage GPUI's GPU capabilities for larger graphs
- **Advanced Routing**: A* algorithm with cost functions for better paths
- **Layout Constraints**: User-defined positioning constraints

## References

- **Implementation**: [ArchViz Layout Crate](../crates/layout/README.md)
- **Examples**: See test sets in `crates/layout/tests/layout.rs`
- **Visualization**: SVG outputs in `crates/layout/doc/`
- **Integration**: Used by `crates/graphview/` for GPUI rendering
