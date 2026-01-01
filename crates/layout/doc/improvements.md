# Layout Engine Performance & Architecture Improvements

This document outlines potential performance and architectural improvements for the archviz-layout engine.

## 🚀 Performance Improvements

### 1. Spatial Partitioning for Force Calculations (High Impact)

**Problem**: Current force-directed algorithm uses O(n²) repulsion calculations, making it slow for large graphs.

**Solution**: Implement spatial partitioning to reduce complexity to O(n log n) or better.

```rust
use std::collections::HashMap;

struct SpatialGrid {
    cell_size: f64,
    cells: HashMap<(i32, i32), Vec<usize>>, // (grid_x, grid_y) -> node indices
}

impl SpatialGrid {
    fn new(nodes: &[Node], cell_size: f64) -> Self {
        let mut cells = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            let grid_x = (node.position.x / cell_size) as i32;
            let grid_y = (node.position.y / cell_size) as i32;
            cells.entry((grid_x, grid_y)).or_insert(Vec::new()).push(i);
        }
        SpatialGrid { cell_size, cells }
    }

    fn nearby_nodes(&self, node_idx: usize, nodes: &[Node], radius: f64) -> Vec<usize> {
        let node = &nodes[node_idx];
        let grid_radius = (radius / self.cell_size).ceil() as i32;
        let center_x = (node.position.x / self.cell_size) as i32;
        let center_y = (node.position.y / self.cell_size) as i32;

        let mut nearby = Vec::new();
        for dx in -grid_radius..=grid_radius {
            for dy in -grid_radius..=grid_radius {
                if let Some(cell_nodes) = self.cells.get(&(center_x + dx, center_y + dy)) {
                    nearby.extend(cell_nodes.iter().filter(|&&i| i != node_idx));
                }
            }
        }
        nearby
    }
}
```

**Expected Impact**: 5-10x speedup for large graphs (n > 100 nodes)

### 2. Parallel Force Calculations (Medium Impact)

**Problem**: Force calculations are computed sequentially.

**Solution**: Use Rayon for parallel processing of independent force calculations.

```rust
use rayon::prelude::*;

fn force_directed_with_spacing_parallel(
    &self,
    nodes: &mut [Node],
    edges: &[(usize, usize, Option<usize>, Option<usize>)],
    spacing: f64,
    prevent_overlaps: bool,
    effective_sizes: &[Size],
    effective_bounds: &[(f64, f64, f64, f64)],
) {
    // Calculate forces in parallel
    let forces: Vec<Position> = (0..nodes.len())
        .into_par_iter()
        .map(|i| self.calculate_node_forces(i, nodes, edges, spacing, effective_sizes))
        .collect();

    // Apply forces sequentially to avoid race conditions
    for i in 0..nodes.len() {
        // ... apply forces logic
    }
}
```

**Expected Impact**: 2-4x speedup on multi-core systems

### 3. Early Termination & Adaptive Iterations (Medium Impact)

**Problem**: Force-directed algorithm always runs for maximum iterations.

**Solution**: Stop iterations when convergence is reached.

```rust
fn force_directed_with_spacing(
    &self,
    nodes: &mut [Node],
    // ... existing params
) {
    let mut prev_energy = f64::INFINITY;
    let mut stable_iterations = 0;
    const MAX_STABLE_ITERATIONS: usize = 10;
    const ENERGY_THRESHOLD: f64 = 0.1;

    for iteration in 0..self.iterations {
        // ... calculate forces

        let current_energy = forces.iter()
            .map(|f| f.x * f.x + f.y * f.y)
            .sum::<f64>();

        if (prev_energy - current_energy).abs() < ENERGY_THRESHOLD {
            stable_iterations += 1;
            if stable_iterations >= MAX_STABLE_ITERATIONS {
                debug!("Converged after {} iterations", iteration);
                break;
            }
        } else {
            stable_iterations = 0;
        }
        prev_energy = current_energy;

        // ... apply forces
    }
}
```

**Expected Impact**: 20-50% reduction in iterations for well-conditioned layouts

### 4. Incremental Layout Updates (High Impact for Interactive Use)

**Problem**: Full layout recomputation for any change.

**Solution**: Cache layout state and only recompute changed parts.

```rust
#[derive(Clone)]
struct LayoutCache {
    node_positions: Vec<Position>,
    effective_bounds: Vec<(f64, f64, f64, f64)>,
    grid: Option<Grid>,
    last_update: std::time::Instant,
}

impl ArchVizLayout {
    fn incremental_layout(
        &self,
        nodes: &mut [Node],
        edges: &[(usize, usize, Option<usize>, Option<usize>)],
        cache: &mut Option<LayoutCache>,
    ) -> LayoutResult {
        // Check if we can reuse cached data
        if let Some(cache) = cache {
            if cache.last_update.elapsed() < std::time::Duration::from_millis(100) {
                // Quick update: only adjust for small changes
                return self.quick_update(nodes, edges, cache);
            }
        }

        // Full layout
        let result = self.layout(nodes.to_vec(), edges.to_vec());

        // Update cache
        *cache = Some(LayoutCache {
            node_positions: result.nodes.iter().map(|n| n.position).collect(),
            effective_bounds: result.nodes.iter().map(|n| n.effective_bounds()).collect(),
            grid: result.grid.clone(),
            last_update: std::time::Instant::now(),
        });

        result
    }
}
```

**Expected Impact**: 10x+ speedup for small changes in interactive scenarios

### 5. Memory Pool for Reused Allocations (Low Impact)

**Problem**: Repeated allocations of temporary vectors.

**Solution**: Pool and reuse allocations.

```rust
#[derive(Default)]
struct LayoutMemoryPool {
    forces_buffer: Vec<Position>,
    overlap_checks: Vec<bool>,
    nearby_nodes: Vec<Vec<usize>>,
}

impl LayoutMemoryPool {
    fn resize_for_nodes(&mut self, node_count: usize) {
        self.forces_buffer.resize(node_count, Position { x: 0.0, y: 0.0 });
        self.overlap_checks.resize(node_count, false);
        self.nearby_nodes.resize(node_count, Vec::new());
    }
}
```

**Expected Impact**: Reduced GC pressure and allocation overhead

### 6. Better Grid Reuse (Medium Impact)

**Problem**: Grid is recreated for each routing phase.

**Solution**: Cache and update grid obstacles instead of full recreation.

```rust
impl Grid {
    fn update_obstacles(&mut self, nodes: &[Node]) {
        // Clear existing obstacles
        for row in &mut self.obstacles {
            for cell in row {
                *cell = false;
            }
        }

        // Recalculate obstacles for current node positions
        // ... existing obstacle calculation logic
    }
}
```

**Expected Impact**: Faster routing for dynamic layouts

## 🏗️ Architectural Improvements

### 7. Modular Algorithm Components (High Impact)

**Problem**: Monolithic layout algorithm hard to extend or customize.

**Solution**: Separate concerns into composable strategies.

```rust
// Add to types.rs
pub trait PlacementStrategy {
    fn place_nodes(&self, nodes: &mut [Node], edges: &[(usize, usize, Option<usize>, Option<usize>)]);
}

pub trait ForceCalculator {
    fn calculate_forces(&self, nodes: &[Node], edges: &[(usize, usize, Option<usize>, Option<usize>)], spacing: f64) -> Vec<Position>;
}

pub trait RoutingStrategy {
    fn route_edges(&self, nodes: &[Node], edges: &[(usize, usize, Option<usize>, Option<usize>)]) -> Vec<Edge>;
}

pub struct ArchVizLayout {
    pub placement: Box<dyn PlacementStrategy>,
    pub forces: Box<dyn ForceCalculator>,
    pub routing: Box<dyn RoutingStrategy>,
    pub iterations: usize,
    // ... other config
}
```

**Benefits**: Enables custom algorithms, easier testing, plugin architecture

### 8. Configuration Profiles (Low Impact)

**Problem**: Fixed configuration doesn't allow quality/performance trade-offs.

**Solution**: Allow different quality/performance profiles.

```rust
#[derive(Clone)]
pub enum LayoutQuality {
    Draft { iterations: usize },      // Fast, lower quality
    Standard { iterations: usize },   // Balanced
    High { iterations: usize },       // Slow, high quality
    Custom { config: ArchVizLayout },
}

impl LayoutQuality {
    pub fn standard() -> Self {
        LayoutQuality::Standard { iterations: 100 }
    }

    pub fn to_config(&self) -> ArchVizLayout {
        match self {
            LayoutQuality::Draft { iterations } => ArchVizLayout {
                iterations: *iterations,
                repulsion_strength: 5000.0, // Weaker forces for speed
                // ... draft settings
            },
            // ... other variants
        }
    }
}
```

**Benefits**: Flexible performance/quality trade-offs

### 9. Constraint-Based Layout (Future Enhancement)

**Problem**: No support for user-defined layout constraints.

**Solution**: Add constraint system for fixed positions, alignments, etc.

```rust
pub enum LayoutConstraint {
    FixedPosition { node_id: String, position: Position },
    HorizontalAlignment { node_ids: Vec<String> },
    VerticalAlignment { node_ids: Vec<String> },
    MinimumDistance { node_a: String, node_b: String, distance: f64 },
    // ... more constraint types
}

struct ConstraintSolver {
    constraints: Vec<LayoutConstraint>,
}

impl ConstraintSolver {
    fn apply_constraints(&self, nodes: &mut [Node]) {
        // Solve constraints using iterative methods
    }
}
```

**Benefits**: Professional layout control, user customization

### 10. Progressive Refinement (Advanced)

**Problem**: All-or-nothing layout computation.

**Solution**: Multi-resolution layout with progressive quality improvement.

```rust
pub struct ProgressiveLayout {
    coarse_layout: ArchVizLayout,
    medium_layout: ArchVizLayout,
    fine_layout: ArchVizLayout,
}

impl ProgressiveLayout {
    pub fn layout_progressive(&self, nodes: Vec<Node>, edges: Vec<(usize, usize, Option<usize>, Option<usize>)>) -> LayoutResult {
        // Phase 1: Very coarse layout (fast)
        let mut result = self.coarse_layout.layout(nodes, edges);

        // Phase 2: Medium refinement
        result = self.medium_layout.layout(result.nodes, /* convert edges */);

        // Phase 3: Fine tuning
        self.fine_layout.layout(result.nodes, /* convert edges */)
    }
}
```

**Benefits**: Responsive UI with quality that improves over time

## 📊 Performance Impact Summary

| Improvement | Expected Speedup | Difficulty | Priority |
|-------------|------------------|------------|----------|
| Spatial partitioning | 5-10x (large graphs) | High | Critical |
| Parallel forces | 2-4x (multi-core) | Medium | High |
| Early termination | 20-50% | Low | High |
| Incremental updates | 10x+ (interactive) | Medium | High |
| Memory pooling | 10-20% | Low | Medium |
| Grid reuse | 20-30% | Low | Medium |

## 🎯 Implementation Roadmap

### Phase 1: Core Performance (High Priority)
1. **Spatial partitioning** - Addresses O(n²) bottleneck
2. **Early termination** - Easy win, immediate benefits
3. **Memory pooling** - Low-hanging fruit

### Phase 2: Scalability (Medium Priority)
1. **Parallel force calculations** - Multi-core utilization
2. **Incremental layout** - Interactive performance
3. **Grid reuse** - Dynamic layout efficiency

### Phase 3: Architecture (Future)
1. **Modular components** - Extensibility framework
2. **Configuration profiles** - Quality/performance trade-offs
3. **Constraint system** - Advanced layout control
4. **Progressive refinement** - Responsive UX

## 🔧 Implementation Notes

- **Dependencies**: Add `rayon` for parallel processing
- **Testing**: Each improvement needs comprehensive benchmarks
- **Backwards Compatibility**: Maintain existing API while adding new features
- **Metrics**: Track layout time, iteration count, memory usage
- **Fallbacks**: Graceful degradation when advanced features aren't available

## 📈 Benchmarking Strategy

```rust
struct LayoutBenchmark {
    node_counts: Vec<usize>,
    edge_counts: Vec<usize>,
    iterations: usize,
}

impl LayoutBenchmark {
    fn run_comprehensive_benchmark(&self) -> BenchmarkResults {
        // Test various graph sizes and configurations
        // Measure: layout time, memory usage, convergence quality
    }
}
```

This roadmap provides a systematic approach to significantly improving layout performance while maintaining code quality and extensibility.
