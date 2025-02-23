# Physics Constraints

AshEngine uses a constraint-based physics system that handles both geometric constraints and collision response for rigid and soft bodies.

For a general overview of constraint systems, see [Constraint Systems Overview](./constraint_systems_overview.md).

## Overview

The constraint system provides:

- Distance constraints
- Volume preservation
- Collision response
- Custom constraint support
- Parallel constraint solving

## Constraint Types

### Base Trait

```rust
pub trait Constraint: Send + Sync {
    fn project(&self, objects: &mut Vec<RefCell<PhysicsObject>>);
    fn clone_box(&self) -> Box<dyn Constraint>;
    fn is_collision_constraint(&self) -> bool;
}
```

### Distance Constraint

Maintains a fixed distance between two points:

```rust
pub struct DistanceConstraint {
    object1_index: usize,
    object2_index: usize,
    rest_distance: f32,
}
```

Usage:

```rust
let constraint = DistanceConstraint::new(
    body1_index,
    body2_index,
    2.0  // rest distance
);
world.add_constraint(Box::new(constraint));
```

### Volume Constraint

Preserves the volume of soft bodies:

```rust
pub struct VolumeConstraint {
    object_index: usize,
    stiffness: f32,
}
```

Features:

- Volume preservation for tetrahedral meshes
- Adjustable stiffness
- Parallel processing support

### Collision Constraint

Handles collision response:

```rust
pub struct CollisionConstraint {
    object1_index: usize,
    object2_index: usize,
    manifold: Option<CollisionManifold>,
    restitution: f32,
    friction: f32,
}
```

## Constraint Solver

### Core Implementation

```rust
pub struct ConstraintSolver {
    pub relaxation: f32,
    pub error_tolerance: f32,
    thread_pool: rayon::ThreadPool,
}
```

Features:

- Parallel constraint resolution
- Iterative solving
- Island-based processing
- SIMD optimization

### Solving Process

1. **Prediction Step**

```rust
// Update positions based on velocities
for object in objects {
    object.predict(delta_time);
}
```

2. **Constraint Iteration**

```rust
for _ in 0..num_iterations {
    // Solve all constraints
    for constraint in constraints {
        constraint.project(objects);
    }
}
```

3. **Velocity Update**

```rust
// Update velocities based on position changes
for object in objects {
    object.update_velocity(delta_time);
}
```

## Island-Based Solving

### Island Structure

```rust
pub struct Island {
    pub objects: Vec<usize>,
    pub constraints: Vec<usize>,
}
```

Benefits:

- Better parallel processing
- Improved cache coherency
- Reduced synchronization overhead

### Implementation

```rust
impl IslandSolver {
    pub fn build_islands(&mut self, world: &PhysicsWorld) -> Vec<Island> {
        // Group connected objects and constraints
        // into independent islands for parallel solving
    }
}
```

## Advanced Features

### 1. Parallel Processing

```rust
fn solve_constraints(&self, world: &mut PhysicsWorld, delta_time: f32) {
    let islands = world.island_solver.build_islands(world);
    let grouped_islands = self.group_islands(&islands);

    // Process island groups in parallel
    self.thread_pool.install(|| {
        grouped_islands.par_iter().for_each(|island_group| {
            self.solve_island_group(island_group, delta_time);
        });
    });
}
```

### 2. XPBD Integration

Extended Position Based Dynamics for better stability:

```rust
fn apply_xpbd_correction(
    &self,
    position: &mut Vec3,
    mass: f32,
    delta_time: f32,
    compliance: f32
) {
    let alpha = compliance / (delta_time * delta_time);
    let correction = /* calculate correction */;
    *position += correction / (1.0 + alpha);
}
```

## Best Practices

### 1. Constraint Setup

- Use appropriate stiffness values
- Consider constraint order
- Balance iteration count

```rust
// Example constraint configuration
world.num_iterations = 10;  // Default
world.constraint_solver.relaxation = 0.2;  // For stability
```

### 2. Performance Optimization

- Group similar constraints
- Use island-based solving
- Implement parallel processing

```rust
// Optimize constraint groups
fn optimize_constraints(&mut self) {
    self.sort_constraints_by_type();
    self.build_constraint_islands();
}
```

### 3. Stability

- Adjust relaxation parameters
- Use sufficient iterations
- Handle edge cases

```rust
// Stability settings
struct StabilitySettings {
    min_iterations: usize,
    max_iterations: usize,
    error_tolerance: f32,
    relaxation: f32,
}
```

## Debug Support

### Visualization

```rust
fn debug_draw_constraints(&self, world: &PhysicsWorld) {
    for constraint in &world.constraints {
        match constraint.type_id() {
            DistanceConstraint::TYPE_ID => {
                // Draw distance constraint
                draw_line(p1, p2, Color::GREEN);
            }
            VolumeConstraint::TYPE_ID => {
                // Draw volume constraint
                draw_tetrahedron(points, Color::BLUE);
            }
            // etc.
        }
    }
}
```

### Profiling

```rust
struct ConstraintProfile {
    solve_time: Duration,
    iteration_count: usize,
    error: f32,
}
```

## Common Issues

1. **Instability**

   - Increase iteration count
   - Adjust relaxation parameter
   - Check constraint ordering

2. **Performance**

   - Use appropriate island sizes
   - Optimize parallel processing
   - Profile constraint solving

3. **Constraint Conflicts**
   - Check constraint compatibility
   - Adjust stiffness values
   - Consider constraint priority

## Future Improvements

1. **Features**

   - Angular constraints
   - Soft constraints
   - Multi-body constraints
   - Constraint graphs

2. **Performance**

   - GPU constraint solving
   - Better parallel algorithms
   - Constraint batching

3. **Integration**
   - Visual constraint editor
   - Runtime constraint modification
   - Constraint serialization
