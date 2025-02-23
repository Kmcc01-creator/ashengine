# Spatial Partitioning

AshEngine uses advanced spatial partitioning techniques to optimize broad-phase collision detection and spatial queries. The system implements both spatial hashing and Morton code-based optimization.

## Overview

The spatial partitioning system provides:

- Efficient broad-phase collision detection
- Spatial hashing with dynamic resizing
- Cache-coherent Morton ordering
- Parallel processing support
- Spatial query capabilities

## Spatial Hash System

### Core Structure

```rust
pub struct SpatialHash {
    cell_size: f32,
    grid: HashMap<GridCell, Vec<usize>>,
    object_cells: Vec<Vec<GridCell>>,
    total_objects: usize,
    cells_used: usize,
}
```

Features:

- Dynamic cell size adjustment
- Load factor monitoring
- Efficient object tracking
- Memory-optimized storage

### Grid Cell

```rust
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GridCell {
    x: i32,
    y: i32,
    z: i32,
}
```

## Cache-Efficient Design

### Morton Encoding

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MortonCode(u64);

impl MortonCode {
    fn new(x: i32, y: i32, z: i32) -> Self {
        let x = Self::expand_bits(x as u64);
        let y = Self::expand_bits(y as u64);
        let z = Self::expand_bits(z as u64);
        MortonCode(x | (y << 1) | (z << 2))
    }

    fn expand_bits(mut v: u64) -> u64 {
        v = (v | (v << 16)) & 0x0000FFFF0000FFFF;
        v = (v | (v << 8)) & 0x00FF00FF00FF00FF;
        v = (v | (v << 4)) & 0x0F0F0F0F0F0F0F0F;
        v = (v | (v << 2)) & 0x3333333333333333;
        v = (v | (v << 1)) & 0x5555555555555555;
        v
    }
}
```

### Cache-Friendly Grid

```rust
pub struct CacheFriendlySpatialHash {
    cell_size: f32,
    grid: HashMap<MortonCode, Vec<usize>>,
}
```

## Dynamic Grid Management

### Adaptive Cell Size

```rust
impl SpatialHash {
    fn calculate_load_factor(&self) -> f32 {
        if self.grid.capacity() == 0 {
            return 0.0;
        }
        self.cells_used as f32 / self.grid.capacity() as f32
    }

    fn resize(&mut self) {
        if self.calculate_load_factor() > LOAD_FACTOR_THRESHOLD {
            self.cell_size *= 2.0; // Double cell size
        } else if self.calculate_load_factor() < LOAD_FACTOR_THRESHOLD / 4.0 {
            self.cell_size /= 2.0; // Halve cell size
        }
    }
}
```

### Object Management

```rust
impl SpatialHash {
    pub fn insert(&mut self, object_index: usize, min: Vec3, max: Vec3) {
        let min_cell = self.position_to_cell(min);
        let max_cell = self.position_to_cell(max);

        self.object_cells[object_index].clear();

        // Insert into all overlapping cells
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    let cell = GridCell { x, y, z };
                    self.insert_to_cell(cell, object_index);
                    self.object_cells[object_index].push(cell);
                }
            }
        }
    }
}
```

## Parallel Broad-Phase

### Implementation

```rust
pub struct ParallelBroadPhase {
    spatial_hash: SpatialHash,
}

impl ParallelBroadPhase {
    pub fn update(&mut self, positions: &[(Vec3, Vec3)]) -> Vec<(usize, usize)> {
        use rayon::prelude::*;

        // Parallel insertion
        positions.par_iter().enumerate().for_each(|(i, (min, max))| {
            self.spatial_hash.insert(i, *min, *max);
        });

        // Get potential pairs
        self.spatial_hash.get_potential_pairs()
    }
}
```

## Spatial Queries

### Radius Query

```rust
impl SpatialHash {
    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<usize> {
        let cells_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.position_to_cell(center);

        // Query cells within radius
        let mut result = std::collections::HashSet::new();
        for x in -cells_radius..=cells_radius {
            for y in -cells_radius..=cells_radius {
                for z in -cells_radius..=cells_radius {
                    let cell = GridCell {
                        x: center_cell.x + x,
                        y: center_cell.y + y,
                        z: center_cell.z + z,
                    };
                    if let Some(objects) = self.grid.get(&cell) {
                        result.extend(objects);
                    }
                }
            }
        }
        result.into_iter().collect()
    }
}
```

## Performance Optimization

### 1. Memory Management

- Preallocated grid cells
- Object cell tracking
- Efficient hash table usage

### 2. Cache Optimization

- Morton code ordering
- Spatial coherence
- Minimal memory fragmentation

### 3. Parallel Processing

- Concurrent object insertion
- Parallel spatial queries
- Thread-safe operations

## Usage Examples

### Basic Usage

```rust
let mut broad_phase = ParallelBroadPhase::new();

// Update object positions
let positions: Vec<(Vec3, Vec3)> = objects.iter()
    .map(|obj| obj.get_bounds())
    .collect();

// Get potential collision pairs
let pairs = broad_phase.update(&positions);

// Process collision pairs
for (i, j) in pairs {
    check_collision(objects[i], objects[j]);
}
```

### Spatial Queries

```rust
// Find objects near a point
let nearby = spatial_hash.query_radius(
    Vec3::new(0.0, 0.0, 0.0),
    10.0
);

// Process nearby objects
for &obj_idx in &nearby {
    process_object(&objects[obj_idx]);
}
```

## Debug Visualization

```rust
fn debug_draw_grid(&self) {
    for (cell, objects) in &self.grid {
        let min = self.cell_to_world(cell);
        let max = min + Vec3::splat(self.cell_size);

        // Draw cell boundaries
        draw_wire_cube(min, max, Color::WHITE);

        // Draw object count
        draw_text(
            &format!("{}", objects.len()),
            cell_center,
            Color::YELLOW
        );
    }
}
```

## Best Practices

1. **Cell Size Selection**

   - Match typical object sizes
   - Consider object distribution
   - Balance grid density

2. **Memory Optimization**

   - Clear unused cells
   - Track object movements
   - Manage grid resizing

3. **Query Optimization**
   - Use appropriate search radius
   - Implement frustum culling
   - Cache query results

## Common Issues

1. **Performance Degradation**

   - Monitor load factor
   - Adjust cell size
   - Profile grid operations

2. **Memory Usage**

   - Clear temporary data
   - Use appropriate capacity
   - Monitor cell distribution

3. **Thread Safety**
   - Use proper synchronization
   - Avoid data races
   - Handle concurrent access

## Future Improvements

1. **Features**

   - Hierarchical grids
   - Dynamic load balancing
   - Adaptive cell sizes

2. **Performance**

   - GPU acceleration
   - Better cache utilization
   - Optimized queries

3. **Integration**
   - Visual debugging tools
   - Performance monitors
   - Scene management
