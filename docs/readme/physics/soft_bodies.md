# Soft Body Physics

AshEngine implements soft body physics using Position-Based Dynamics (PBD), allowing for realistic simulation of deformable objects, cloth, and soft bodies.

## Features

- Tetrahedral mesh-based deformation
- Volume preservation
- Shape matching
- Particle-based simulation
- Constraint-based dynamics
- Efficient parallel processing

## Implementation

### Data Structure

```rust
PhysicsObject::DeformableBody {
    positions: Vec<Vec3>,        // Current particle positions
    prev_positions: Vec<Vec3>,   // Previous positions for velocity computation
    velocities: Vec<Vec3>,       // Particle velocities
    masses: Vec<f32>,           // Per-particle masses
    rest_volume: f32,           // Initial volume for preservation
    volumes: Vec<f32>,          // Current tetrahedral volumes
    tetrahedra: Vec<[usize; 4]>, // Tetrahedral mesh connectivity
    bounding_box: Vec4,         // Overall bounding box
}
```

### Components

1. **Particle System**

   - Position and velocity for each particle
   - Mass distribution
   - Parallel updates for performance

2. **Tetrahedral Mesh**

   - Four-point tetrahedra for volume representation
   - Volume preservation constraints
   - Shape maintenance

3. **Constraint System**
   - Distance constraints between particles
   - Volume preservation
   - Shape matching constraints

## Position-Based Dynamics

The PBD algorithm follows these steps:

1. **Prediction**

   ```rust
   for particle in particles {
       particle.prev_position = particle.position;
       particle.velocity += gravity * delta_time;
       particle.position += particle.velocity * delta_time;
   }
   ```

2. **Constraint Projection**

   ```rust
   for _ in 0..num_iterations {
       // Project distance constraints
       project_distance_constraints();

       // Project volume constraints
       project_volume_constraints();

       // Project collision constraints
       project_collision_constraints();
   }
   ```

3. **Velocity Update**
   ```rust
   for particle in particles {
       particle.velocity =
           (particle.position - particle.prev_position) / delta_time;
   }
   ```

## Volume Preservation

### Tetrahedral Volume Calculation

```rust
fn calculate_tetrahedron_volume(p1: Vec3, p2: Vec3, p3: Vec3, p4: Vec3) -> f32 {
    let v1 = p2 - p1;
    let v2 = p3 - p1;
    let v3 = p4 - p1;
    (v1.cross(v2).dot(v3)).abs() / 6.0
}
```

### Volume Constraint

The volume preservation constraint ensures the total volume remains close to the rest volume:

```rust
let scale = (rest_volume / current_volume).cbrt();
for pos in positions {
    *pos = center + (*pos - center) * scale;
}
```

## Usage

### Creating a Soft Body

```rust
// Create particle positions, masses, and tetrahedral mesh
let positions: Vec<Vec3> = /* particle positions */;
let masses: Vec<f32> = /* particle masses */;
let tetrahedra: Vec<[usize; 4]> = /* tetrahedral mesh */;

let soft_body = PhysicsObject::new_deformable_body(
    positions,
    masses,
    tetrahedra,
    1.0, // bounding size
);
world.add_object(soft_body);
```

### Adding Constraints

```rust
// Add distance constraints between particles
for &[i1, i2, _, _] in &tetrahedra {
    world.add_constraint(Box::new(DistanceConstraint::new(
        i1, i2, rest_length
    )));
}

// Add volume preservation
world.add_constraint(Box::new(VolumeConstraint::new(
    body_index,
    0.5 // stiffness
)));
```

## Optimization

1. **Parallel Processing**

   - SIMD operations for particle updates
   - Parallel constraint solving
   - Batch processing for performance

2. **Memory Layout**

   - Cache-friendly particle storage
   - Efficient constraint representation
   - Optimized tetrahedral mesh structure

3. **Constraint Solving**
   - Iterative solver with relaxation
   - Island-based parallel processing
   - Adaptive iteration count

## Best Practices

1. **Mesh Generation**

   - Use high-quality tetrahedral meshes
   - Balance resolution vs performance
   - Ensure proper connectivity

2. **Constraint Setup**

   - Use appropriate stiffness values
   - Balance number of constraints
   - Consider performance impact

3. **Performance Tuning**
   - Adjust iteration count based on needs
   - Use appropriate particle counts
   - Configure constraint groups efficiently

## Debug Visualization

The system supports various debug visualizations:

- Particle positions
- Tetrahedral mesh
- Constraint networks
- Volume preservation forces

## Common Issues

1. **Volume Loss**

   - Increase volume constraint stiffness
   - Add more constraint iterations
   - Check for inverted tetrahedra

2. **Instability**

   - Reduce timestep
   - Increase iteration count
   - Adjust constraint weights

3. **Performance**
   - Optimize mesh resolution
   - Use appropriate particle counts
   - Balance constraint complexity

## Examples

### Basic Soft Body

```rust
// Create a simple cube soft body
let (positions, tetrahedra) = create_cube_mesh(1.0);
let masses = vec![1.0; positions.len()];

let soft_body = PhysicsObject::new_deformable_body(
    positions,
    masses,
    tetrahedra,
    1.0
);

world.add_object(soft_body);
```

### Complex Constraints

```rust
// Add shape matching for better stability
let shape_match = ShapeMatchingConstraint::new(
    body_index,
    rest_positions.clone(),
    0.5 // stiffness
);
world.add_constraint(Box::new(shape_match));

// Add volume preservation
let volume = VolumeConstraint::new(
    body_index,
    0.8 // stiffness
);
world.add_constraint(Box::new(volume));
```

## Future Improvements

1. **Features**

   - Cloth simulation
   - Two-way coupling with rigid bodies
   - Better volume preservation
   - Plasticity and fracture

2. **Performance**

   - GPU acceleration
   - Better parallel processing
   - Optimized constraint solving

3. **Integration**
   - Better graphics system integration
   - Enhanced debug visualization
   - Real-time parameter tuning
