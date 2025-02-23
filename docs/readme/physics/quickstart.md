# GPU Physics Quick Start Guide

This guide demonstrates how to quickly integrate and use the GPU physics system in your application.

## Basic Integration

```rust
use ashengine::physics::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Vulkan components (device, queue, etc.)
    // ... [Your Vulkan initialization code here]

    // 2. Configure the physics system
    let config = PhysicsConfig {
        initial_particles: 10_000,   // Start with 10k particles
        debug_enabled: true,         // Enable debugging
        debug_sample_rate: 60,       // Update stats every 60 frames
        initial_pool_size: 4 * 1024 * 1024,  // 4MB initial memory pool
        max_recovery_attempts: 3,    // Number of recovery attempts on error
    };

    // 3. Create the physics system
    let (mut physics, mut debug) = create_physics_system(
        device.clone(),
        physical_device,
        queue_family_index,
        Some(config),
    )?;

    // 4. Create initial particles
    let mut particles = vec![
        Particle {
            position: [0.0, 0.0, 0.0, 1.0],
            velocity: [1.0, 1.0, 1.0, 0.0],
        };
        10_000
    ];

    // 5. Main update loop
    loop {
        // Update particle data on GPU
        physics.update_particle_data(&particles)?;

        // Configure simulation parameters
        let push_constants = PushConstants {
            delta_time: 0.016,  // 60 FPS
            max_velocity: 10.0,
            bounds: [-100.0, 100.0],
        };

        // Record and submit compute commands
        physics.record_compute_commands(push_constants)?;
        physics.submit_compute()?;

        // Wait for compute to finish
        physics.wait_for_compute()?;

        // Read back results
        particles = physics.read_particle_data(10_000)?;

        // Debug visualization
        if debug.should_update() {
            debug.update_stats(
                &particles,
                physics.get_profiling_data().last_compute_time,
                [-100.0, 100.0],
                10.0
            );
            println!("{}", debug.get_stats_string());
        }

        // Optional: Check memory usage
        let memory_stats = physics.get_memory_stats();
        println!(
            "Memory Usage: {}/{} MB",
            memory_stats.used_size / (1024 * 1024),
            memory_stats.total_size / (1024 * 1024)
        );
    }
}
```

## Error Handling Example

```rust
fn handle_physics_update(
    physics: &mut GpuPhysicsSystem,
    particles: &[Particle],
    push_constants: PushConstants,
) -> Result<(), PhysicsError> {
    match physics.update_particle_data(particles) {
        Ok(_) => (),
        Err(PhysicsError::DeviceLost(msg)) => {
            println!("Device lost: {}. Attempting recovery...", msg);
            physics.try_recover()?;
            return physics.update_particle_data(particles);
        }
        Err(PhysicsError::BufferOverflow(msg)) => {
            println!("Buffer overflow: {}. Resizing...", msg);
            physics.resize(particles.len() * 2)?;
            return physics.update_particle_data(particles);
        }
        Err(e) => return Err(e),
    }

    physics.record_compute_commands(push_constants)?;
    physics.submit_compute()?;

    Ok(())
}
```

## Dynamic Resizing Example

```rust
fn handle_particle_spawn(
    physics: &mut GpuPhysicsSystem,
    particles: &mut Vec<Particle>,
    new_particles: &[Particle],
) -> Result<(), PhysicsError> {
    // Check if we need to resize
    if particles.len() + new_particles.len() > particles.capacity() {
        let new_size = (particles.len() + new_particles.len()).next_power_of_two();
        physics.resize(new_size)?;
    }

    // Add new particles
    particles.extend_from_slice(new_particles);

    Ok(())
}
```

## Debug Visualization Example

```rust
fn monitor_performance(
    physics: &GpuPhysicsSystem,
    debug: &mut DebugVisualization,
    particles: &[Particle],
) {
    if debug.should_update() {
        debug.update_stats(
            particles,
            physics.get_profiling_data().last_compute_time,
            [-100.0, 100.0],
            10.0
        );

        // Print performance stats
        println!("=== Performance Stats ===");
        println!("{}", debug.get_stats_string());

        // Check for performance warnings
        let stats = debug.get_stats();
        if stats.compute_time.as_secs_f32() > 0.016 {
            println!("Warning: Frame time exceeds 16ms target");
        }
        if stats.max_velocity_violations > 0 {
            println!("Warning: {} particles exceeding max velocity",
                stats.max_velocity_violations);
        }
    }
}
```

## Best Practices

1. **Initialization**

   - Configure appropriate initial particle count
   - Enable debug visualization during development
   - Set reasonable memory pool size

2. **Error Handling**

   - Always handle potential errors
   - Implement recovery strategies
   - Monitor system state

3. **Performance**

   - Use debug visualization to monitor performance
   - Resize buffers efficiently
   - Clean up resources properly

4. **Memory Management**
   - Monitor memory usage
   - Pre-allocate enough space
   - Use buffer pooling

See the [Performance Guide](performance_guide.md) for detailed optimization strategies.
