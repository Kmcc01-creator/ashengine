# GPU-Accelerated Particle Physics System

## Overview

The ashengine GPU physics system provides high-performance particle simulation using Vulkan compute shaders. The system is designed for efficiency, stability, and ease of use, featuring:

- Double-buffered particle data for optimal GPU-CPU synchronization
- Memory pooling and dynamic buffer resizing
- Comprehensive debug visualization and profiling
- Robust error handling and recovery mechanisms

## Quick Start

```rust
use ashengine::physics::prelude::*;

// Configure the physics system
let config = PhysicsConfig {
    initial_particles: 10000,
    debug_enabled: true,
    debug_sample_rate: 60,
    initial_pool_size: 4 * 1024 * 1024, // 4MB
    max_recovery_attempts: 3,
};

// Create physics system with debug support
let (mut physics, mut debug) = create_physics_system(
    device,
    physical_device,
    queue_family_index,
    Some(config),
)?;

// Main update loop
physics.update_particle_data(&particles)?;
physics.record_compute_commands(push_constants)?;
physics.submit_compute()?;

// Get debug information
if debug.should_update() {
    debug.update_stats(
        &particles,
        physics.get_profiling_data().last_compute_time,
        [-100.0, 100.0],
        10.0
    );
    println!("{}", debug.get_stats_string());
}
```

## Core Features

### Memory Management

The system uses a sophisticated memory management approach:

- **Memory Pooling**: Efficient allocation and reuse of GPU memory
- **Dynamic Resizing**: Automatic buffer growth and shrinking
- **Persistent Mapping**: Zero-copy updates for optimal performance
- **Memory Type Selection**: Automatic selection of optimal memory types

```rust
// Memory stats and monitoring
let stats = physics.get_memory_stats();
println!("Memory Usage: {}/{} bytes", stats.used_size, stats.total_size);
println!("Free Blocks: {}", stats.free_block_count);
```

### Double Buffering

The system implements double buffering for efficient GPU-CPU synchronization:

- **Front Buffer**: Current frame data for GPU computation
- **Back Buffer**: Next frame data for CPU updates
- **Automatic Swapping**: Managed internally by the system

### Debug Support

Comprehensive debugging and profiling capabilities:

```rust
// Enable debug visualization
debug.enable();

// Get detailed statistics
let stats = debug.get_stats();
println!("Active Particles: {}", stats.active_particles);
println!("Compute Time: {:.2}ms", stats.compute_time.as_secs_f32() * 1000.0);
println!("Bounds Violations: {:?}", stats.bounds_violations);
```

### Error Handling

Robust error handling and recovery system:

```rust
match physics.submit_compute() {
    Ok(_) => println!("Compute completed successfully"),
    Err(PhysicsError::DeviceLost(msg)) => {
        println!("Device lost: {}", msg);
        physics.try_recover()?;
    }
    Err(e) => println!("Error: {}", e),
}
```

## Performance Optimization

### Memory Layout

Particle data is structured for optimal GPU access:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    position: [f32; 4],  // Aligned for GPU vec4
    velocity: [f32; 4],  // Aligned for GPU vec4
}
```

### Compute Shader Optimization

The compute shader utilizes:

- Shared memory for coalesced access
- Vectorized operations
- Efficient branching

### Memory Barriers

Proper synchronization ensures data consistency:

- Host to device transfers
- Device to host transfers
- Between compute passes

## Configuration

### Physics Configuration

```rust
let config = PhysicsConfig {
    // Number of particles to pre-allocate
    initial_particles: 10000,

    // Enable debug features
    debug_enabled: true,

    // Update debug stats every N frames
    debug_sample_rate: 60,

    // Initial GPU memory pool size
    initial_pool_size: 4 * 1024 * 1024,

    // Error recovery attempts
    max_recovery_attempts: 3,
};
```

### Debug Visualization

```rust
// Configure debug visualization
debug.set_sample_rate(60);  // Update every 60 frames
debug.enable();             // Enable visualization

// Update stats
debug.update_stats(
    &particles,
    compute_time,
    bounds,
    max_velocity
);

// Get formatted stats
println!("{}", debug.get_stats_string());
```

## Best Practices

1. **Memory Management**

   - Pre-allocate enough particles to avoid frequent resizing
   - Monitor memory usage with `get_memory_stats()`
   - Clean up resources properly using `Drop` trait

2. **Performance**

   - Use persistent mapped buffers for updates
   - Batch particle updates when possible
   - Monitor compute times through debug stats

3. **Error Handling**
   - Always check for errors after operations
   - Implement proper recovery mechanisms
   - Monitor system state for stability

## Future Improvements

Planned improvements include:

- Advanced workgroup size optimization
- Multi-GPU support
- Additional physics features (cloth, fluids)
- Enhanced profiling tools

## Technical Details

### Memory Pool Configuration

The memory pool system uses the following defaults:

- Minimum Pool Size: 1MB
- Maximum Pool Size: 256MB
- Default Block Size: 64KB

These can be adjusted based on your needs:

```rust
const MIN_POOL_SIZE: u64 = 1024 * 1024;      // 1MB
const MAX_POOL_SIZE: u64 = 256 * 1024 * 1024; // 256MB
```

### Profiling Data

The profiling system collects:

- Compute time per frame
- Memory allocation statistics
- Particle statistics
- Performance metrics

### Debug Statistics

Available debug statistics include:

- Active particle count
- Particles in/out of bounds
- Velocity violations
- Average positions/velocities
- Compute time metrics
- Memory usage details
