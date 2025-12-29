# Custom Layout Algorithm

This document outlines a custom graph layout algorithm designed for architecture visualization with the following requirements.

## Requirements

- **Variable Node Sizes**: Nodes have different rectangular dimensions that must be respected during layout.
- **Node Ports**: Nodes have ports for edge connections with different modes:
  - Single port per node
  - Input ports on left, output ports on right
  - Up to 8 ports distributed around the rectangle sides (configurable as input/output)
  - When multiple ports on a side, they are equally spaced along that side
  - Ports can be reused for multiple edges if more connections than ports exist
- **Port-Based Routing**: Edges connect to specific ports; routing chooses ports on the side facing the target node direction.
- **Orthogonal Edge Routing**: Edges use Manhattan-style routing with 90-degree bends and multiple segments (like electronic schematics).
- **Minimize Overlaps and Crossings**: Prevent node overlaps and reduce edge crossings for clarity.
- **Compact Layout**: Produce a dense layout that fits well on paper while maintaining sufficient spacing for edge routing.
- **Edge Overlap Tolerance**: Edges can overlap each other (highlighted on selection), but provide an option for more spacing to manually trace edges.
- **Non-Hierarchical**: No assumption of hierarchical structure; handles highly interconnected graphs.
- **Performance**: Efficient for graphs with 50-200 nodes typical in architecture diagrams.

## Algorithm Draft

### Phase 1: Initial Placement
- Analyze graph connectivity to identify clusters of densely connected nodes.
- Place nodes using a combination of size and connectivity:
  - Group nodes with high shared connections into spatial clusters.
  - Within clusters, use shelf-packing to arrange nodes by size (largest first).
  - Position clusters to minimize inter-cluster edge lengths.
- Alternative: Use spectral layout or community detection to seed initial positions.

### Phase 2: Force-Directed Refinement
- Apply force-directed simulation with size-aware forces:
  - **Repulsion**: Nodes repel each other based on their bounding boxes (distance between closest edges).
  - **Attraction**: Connected nodes attract along their connecting edges.
  - **Size Constraints**: Prevent nodes from overlapping by treating them as non-point masses.
- Use a cooling schedule to stabilize the layout.
- Run for fixed iterations or until convergence.

### Phase 3: Edge Routing
- For each edge, assign ports on source and target nodes:
  - Select ports based on direction: choose ports on the side facing the target node.
  - Reuse ports if necessary for multiple edges.
- Compute path from source port to target port with Manhattan routing:
  - Ensure paths avoid routing through node interiors (obstacle avoidance).
  - Always use multi-segment paths with 90-degree bends; direct paths are forbidden.
  - For dense layouts, allow edge overlaps; for spaced layouts, maintain minimum clearances.

### Phase 4: Calculate Canvas and Center Layout
- Compute the bounding box of all nodes and edges.
- Determine minimum canvas size with padding (default 400x300 minimum + spacing).
- Translate the entire layout to center it on the canvas, ensuring top-left coordinates start from (0,0) as in game industry standards.

### Phase 5: Optimization
- **Crossing Reduction**: Swap node positions within local neighborhoods to reduce edge crossings.
- **Compaction**: Apply additional forces to pull nodes closer while maintaining minimum spacing.
- **Edge Spacing Option**: If enabled, increase node spacing and reroute edges to avoid overlaps.

## Implementation Considerations

- **Data Structures**: Use quadtree for efficient neighbor queries during force calculation.
- **Performance**: Limit iterations; use GPU acceleration if available (fits GPUI context).
- **Parameters**: Configurable repulsion strength, attraction strength, minimum spacing, edge overlap tolerance.
- **Hybrid Approach**: Combine with existing algorithms (e.g., force-directed for initial layout, then custom routing).
- **Evaluation**: Measure layout quality by node overlap area, total edge length, number of crossings.

## Potential Challenges

- Balancing compactness with readability.
- Handling very large size differences between nodes.
- Ensuring orthogonal routing doesn't create excessive bends.
- Scaling to larger graphs (>500 nodes).

## References

- Force-directed: Fruchterman-Reingold algorithm
- Packing: Shelf algorithms for rectangle packing
- Routing: Orthogonal graph drawing techniques
- Open source inspiration: D3.js force layout, ELK framework
