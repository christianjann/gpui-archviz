# ELK (Eclipse Layout Kernel)

ELK is a layout framework developed by Eclipse that provides various graph layout algorithms, including hierarchical layouts similar to Sugiyama. It offers modular algorithms that can be combined for different layout needs.

## How It Works

ELK uses a pipeline approach with configurable phases:

1. **Graph Analysis**: Analyzes graph structure and properties.

2. **Layering**: Assigns nodes to layers using algorithms like:
   - Longest Path
   - Coffman-Graham
   - Network Simplex

3. **Crossing Minimization**: Reduces edge crossings within and between layers using:
   - Barycenter method
   - Median method
   - Port constraints

4. **Node Placement**: Positions nodes within layers with spacing algorithms.

5. **Edge Routing**: Routes edges with options for:
   - Orthogonal routing
   - Spline routing
   - Polyline routing

## Key Features

- Highly configurable with many options
- Supports various graph types (hierarchical, compound, etc.)
- Modular architecture for custom layouts
- Good performance on large graphs
- Integration with Eclipse and other tools

## Advantages

- Flexible and extensible
- High-quality layouts
- Supports complex constraints
- Active development and maintenance

## Disadvantages

- Complex configuration
- Steeper learning curve
- Java-based (though has JavaScript ports)

## Open Source Implementations

- [ELK (Java)](https://github.com/eclipse/elk): Main implementation
- [elkjs (JavaScript)](https://github.com/kieler/elkjs): JavaScript port
- [Dagre](https://github.com/dagrejs/dagre): Similar hierarchical layout
- [Graphviz](https://graphviz.org/): Related layout tools

## References

- Eclipse Layout Kernel: https://www.eclipse.org/elk/
- Spönemann, M., Köhler, M., & von Hanxleden, R. (2016). ELK: The Eclipse Layout Kernel. In International Conference on Graph Drawing and Network Visualization (pp. 495-497). Springer.
