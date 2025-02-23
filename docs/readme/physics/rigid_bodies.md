# Rigid Body Physics

The rigid body system in AshEngine provides a complete implementation of 6-DOF (degrees of freedom) physics simulation for solid objects.

## Features

- Full 3D motion with position and orientation
- Linear and angular velocity/acceleration
- Mass and inertia tensor support
- Collision response with friction and restitution
- Constraint-based interactions

## Implementation

### Data Structure

```rust
PhysicsObject::RigidBody {
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    orientation: Quat,
    angular_velocity: Vec3,
    angular_acceleration: Vec3,
    mass: f32,
    inertia_tensor: Vec3,
    bounding_box: Vec4,
}
```

### Components

1. **Position & Orientation**

   - `position`: Center of mass in world space
   - `orientation`: Quaternion representing rotation
   - Automatically updated during simulation

2. **Motion**

   - `velocity`: Linear velocity vector
   - `angular_velocity`: Angular velocity vector
   - Applied forces and torques affect these values

3. **Physical Properties**

   - `mass`: Object mass (affects motion)
   - `inertia_tensor`: Rotational inertia (simplified as diagonal)
   - Used in collision response and constraint solving

4. **Collision**
   - `bounding_box`: AABB for broad-phase collision
   - Oriented bounding box for narrow-phase
   - Contact point generation for accurate response

## Usage

### Creating a Rigid Body

```rust
let rigid_body = PhysicsObject::new_rigid_body(
    Vec3::new(0.0, 10.0, 0.0), // Initial position
    1.0,                        // Mass
    Vec4::splat(1.0),          // Bounding box (w component is size)
);
world.add_object(rigid_body);
```

### Applying Forces

```rust
match &mut *object.borrow_mut() {
    PhysicsObject::RigidBody {
        acceleration,
        angular_acceleration,
        ..
    } => {
        *acceleration += force / mass;
        *angular_acceleration += torque / inertia;
    }
    _ => {}
}
```

### Collision Response

The system automatically handles:

- Contact point generation
- Impulse-based collision response
- Friction and restitution effects
- Angular response from off-center collisions

## Integration Method

The rigid body simulation uses a semi-implicit Euler integration scheme:

1. **Velocity Update**

   ```rust
   velocity += acceleration * delta_time;
   angular_velocity += angular_acceleration * delta_time;
   ```

2. **Position Update**

   ```rust
   position += velocity * delta_time;
   ```

3. **Orientation Update**
   ```rust
   let angle = angular_velocity.length() * delta_time;
   if angle != 0.0 {
       let axis = angular_velocity / angle;
       orientation = Quat::from_axis_angle(axis, angle) * orientation;
   }
   ```

## Optimization

1. **Memory Layout**

   - Contiguous memory for better cache utilization
   - SIMD-friendly data structures
   - Efficient quaternion operations

2. **Collision Detection**

   - Hierarchical broad-phase using spatial hashing
   - Optimized narrow-phase with SAT algorithm
   - Early-out checks for performance

3. **Constraint Solving**
   - Parallel constraint resolution
   - Island-based processing
   - Iterative velocity solving

## Best Practices

1. **Mass and Inertia**

   - Use realistic mass values
   - Configure inertia tensor appropriately
   - Avoid extreme mass ratios

2. **Collision Shape**

   - Use accurate bounding boxes
   - Consider using compound shapes for complex objects
   - Balance precision vs performance

3. **Stability**
   - Adjust timestep for stability
   - Use appropriate constraint iterations
   - Configure damping parameters

## Examples

### Basic Movement

```rust
// Create a rigid body
let body = PhysicsObject::new_rigid_body(
    Vec3::new(0.0, 0.0, 0.0),
    1.0,
    Vec4::splat(1.0)
);

// Add to world
world.add_object(body);

// Apply force
if let PhysicsObject::RigidBody { acceleration, .. } = &mut *body.borrow_mut() {
    *acceleration += Vec3::new(0.0, 10.0, 0.0); // Apply upward force
}
```

### Constraint Usage

```rust
// Create two rigid bodies
let body1 = PhysicsObject::new_rigid_body(p1, m1, bb1);
let body2 = PhysicsObject::new_rigid_body(p2, m2, bb2);

// Add to world
let body1_idx = world.add_object(body1);
let body2_idx = world.add_object(body2);

// Create a distance constraint between them
let constraint = DistanceConstraint::new(body1_idx, body2_idx, 2.0);
world.add_constraint(Box::new(constraint));
```

## Debugging

The physics system supports debug visualization:

- Bounding box visualization
- Velocity vectors
- Contact points
- Constraint visualization

## Common Issues

1. **Instability**

   - Solution: Reduce timestep or increase iteration count
   - Check mass ratios and constraint parameters
   - Consider using more substeps

2. **Performance**

   - Use appropriate broad-phase settings
   - Optimize collision shapes
   - Balance accuracy vs speed with iteration counts

3. **Collision Issues**
   - Verify bounding box sizes
   - Check collision margins
   - Ensure proper contact generation
