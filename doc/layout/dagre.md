# Dagre Layout (Sugiyama Algorithm)

Dagre is a JavaScript library that implements hierarchical graph layout using the Sugiyama algorithm. It arranges nodes in layers (ranks) with edges flowing in one direction, typically top-to-bottom.

## How It Works

1. **Cycle Removal**: Detects and reverses edges to make the graph acyclic.

2. **Layer Assignment**: Assigns nodes to layers (ranks) based on longest path from sources.

3. **Crossing Reduction**: Reorders nodes within layers to minimize edge crossings.

4. **Position Assignment**: Positions nodes within layers and routes edges orthogonally.

5. **Edge Routing**: Draws edges with minimal bends, often using splines or polylines.

## Key Features

- Handles large graphs efficiently
- Supports edge labels and ports
- Customizable node and edge spacing
- Orthogonal edge routing

## Advantages

- Produces clear hierarchical layouts
- Good for flowcharts and organization charts
- Deterministic results

## Disadvantages

- Assumes hierarchical structure
- May not work well for non-hierarchical graphs
- Can be slow for very large graphs

## Open Source Implementations

- [Dagre (JavaScript)](https://github.com/dagrejs/dagre): Original implementation
- [dagre-rs (Rust)](https://github.com/fschutt/dagre-rs): Rust port used in this project
- [Graphviz dot](https://graphviz.org/docs/layouts/dot/): C implementation
- [ELK (Eclipse Layout Kernel)](https://www.eclipse.org/elk/): Java implementation with similar algorithms

## References

- Sugiyama, K., Tagawa, S., & Toda, M. (1981). Methods for visual understanding of hierarchical system structures. IEEE Transactions on Systems, Man, and Cybernetics, 11(2), 109-125.
- Gansner, E. R., Koutsofios, E., North, S. C., & Vo, K. P. (1993). A technique for drawing directed graphs. IEEE Transactions on Software Engineering, 19(3), 214-230.
