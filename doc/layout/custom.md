# Custom ArciVis Layout

Custom layout algorithm developed for vehicle E/E architecture diagrams, combining force-directed positioning with orthogonal edge routing.

## How It Works

1. **Initial Placement**: Nodes placed in grid based on connectivity.
2. **Force-Directed Refinement**: Repulsion and attraction forces optimize positions.
3. **Edge Routing**: Grid-based pathfinding with obstacle avoidance.
4. **Finalization**: Canvas calculation and centering.

## Key Features

- Obstacle-aware orthogonal routing
- Port-aware connections
- Variable node sizes
- Configurable parameters

## Advantages

- Optimized for architecture diagrams
- Clean orthogonal edges
- Handles complex connectivity

## Disadvantages

- Computationally intensive for very large graphs
- Requires tuning for specific use cases

## Implementation

- [ArciVis Layout Crate](https://github.com/christianjann/arcivis-layout)

## References

- See crate documentation for detailed API and examples.
