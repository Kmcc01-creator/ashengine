# Collision Detection

AshEngine implements a comprehensive collision detection system that handles rigid-rigid, soft-soft, and rigid-soft body collisions using a two-phase approach.

## Overview

The collision system consists of:

- Broad-phase: Quick elimination of non-colliding pairs
- Narrow-phase: Precise collision detection and contact generation
- Contact manifold generation
- Support for multiple collision types

## Broad-Phase Detection

### Spatial Hashing

```rust
pub struct SpatialHash {
    cell_size: f32,
    grid: HashMap<GridCell, Vec<usize>>,
    object_cells: Vec<Vec<GridCell>>,
}
```

Features:

- Dynamic cell size adjustment
- Morton code ordering for cache efficiency
- Parallel processing support
- O(n) average case complexity

### Implementation

```rust
// Insert object into spatial hash
let min_cell = position_to_cell(min_bound);
let max_cell = position_to_cell(max_bound);

for x in min_cell.x..=max_cell.x {
    for y in min_cell.y..=max_cell.y {
        for z in min_cell.z..=max_cell.z {
            grid.insert(GridCell { x, y, z }, object_index);
        }
    }
}
```

## Narrow-Phase Detection

### Rigid Body Collision

Uses Separating Axis Theorem (SAT) for oriented bounding boxes:

```rust
fn detect_rigid_rigid_collision(
    p1: &Vec3,
    o1: &Quat,
    bb1: &Vec4,
    p2: &Vec3,
    o2: &Quat,
    bb2: &Vec4,
) -> Option<CollisionManifold>
```

Features:

- Full 3D oriented box collision
- Contact point generation
- Penetration depth calculation
- Normal direction computation

### Soft Body Collision

#### Particle-Based Detection

```rust
fn detect_soft_soft_collision(
    p1s: &[Vec3],
    t1s: &[[usize; 4]],
    p2s: &[Vec3],
    t2s: &[[usize; 4]],
) -> Option<CollisionManifold>
```

Features:

- Tetrahedral mesh intersection
- Particle proximity checks
- Volume-based collision response
- Multiple contact point generation

### Mixed Collision Types

Handles collisions between different object types:

- Rigid vs. Soft body
- Point vs. Triangle tests
- Continuous collision detection (planned)

## Contact Generation

### Collision Manifold

```rust
pub struct CollisionManifold {
    pub normal: Vec3,
    pub penetration: f32,
    pub contact_points: Vec<Vec3>,
}
```

Features:

- Multiple contact points
- Penetration depth
- Collision normal
- Relative velocity at contact

### Contact Point Reduction

```rust
fn reduce_contact_points(points: &[Vec3], max_points: usize) -> Vec<Vec3> {
    // Clustering algorithm to reduce contact points
    // while maintaining good contact coverage
}
```

## Optimization Techniques

### 1. Broad-Phase Optimization

- Dynamic grid size adjustment

```rust
fn resize_grid(&mut self) {
    if self.calculate_load_factor() > THRESHOLD {
        self.cell_size *= 2.0;
    }
}
```

- Morton code spatial coherence

```rust
fn morton_encode(x: u32, y: u32, z: u32) -> u64 {
    // Interleave bits for better cache usage
}
```

### 2. Narrow-Phase Optimization

- SIMD operations for multiple tests
- Early-out checks
- Cached calculations
- Parallel processing of collision pairs

## Usage Examples

### Basic Collision Detection

```rust
// Create spatial hash for broad-phase
let mut broad_phase = ParallelBroadPhase::new();

// Update object positions
let aabb_pairs = world.gather_aabb_pairs();
let potential_collisions = broad_phase.update(&aabb_pairs);

// Perform narrow-phase detection
for (i, j) in potential_collisions {
    if let Some(manifold) = detect_collision(&objects[i], &objects[j]) {
        // Handle collision response
    }
}
```

### Custom Collision Handlers

```rust
// Implementing custom collision detection
impl CollisionHandler for CustomObject {
    fn detect_collision(&self, other: &dyn CollisionObject) -> Option<CollisionManifold> {
        // Custom collision detection logic
    }
}
```

## Debug Visualization

The system provides visualization for:

- Spatial grid cells
- Collision pairs
- Contact points and normals
- Penetration depths

```rust
fn debug_draw_collision(manifold: &CollisionManifold) {
    // Draw contact points
    for point in &manifold.contact_points {
        draw_point(*point, Color::RED);
    }

    // Draw normal
    draw_arrow(
        manifold.contact_points[0],
        manifold.normal * manifold.penetration
    );
}
```

## Performance Considerations

### 1. Broad-Phase Tuning

- Adjust cell size based on object sizes
- Balance grid density vs. lookup cost
- Use appropriate spatial coherence techniques

### 2. Narrow-Phase Optimization

- Implement efficient primitive tests
- Use hierarchical bounding volumes
- Optimize contact generation

### 3. Memory Management

- Pool allocators for contact points
- Efficient manifold storage
- Temporary allocation reduction

## Common Issues

1. **Tunneling**

   - Solution: Implement continuous collision detection
   - Use smaller timesteps
   - Increase substep count

2. **Performance Bottlenecks**

   - Profile broad-phase vs. narrow-phase
   - Optimize spatial partitioning
   - Reduce unnecessary collision checks

3. **Stability Issues**
   - Improve contact point generation
   - Adjust collision margins
   - Fine-tune response parameters

## Future Improvements

1. **Features**

   - Continuous collision detection
   - Convex hull support
   - Better triangle mesh collisions
   - GPU-accelerated broad-phase

2. **Performance**

   - Enhanced parallel processing
   - Better cache utilization
   - Optimized contact generation

3. **Integration**
   - Physics middleware support
   - Better debugging tools
   - Performance profiling tools
