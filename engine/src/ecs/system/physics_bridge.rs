//! Bridge system between ECS and GPU physics
//!
//! This system synchronizes physics components with the GPU physics system,
//! handling data transfer and state updates.

use glam::Vec3;
use std::sync::Arc;

use super::{System, SystemStage};
use crate::ecs::component::{PhysicsComponent, TransformComponent};
use crate::ecs::World;
use crate::physics::{GpuPhysicsSystem, Particle, PhysicsError};

/// System that bridges ECS physics components with the GPU physics system
pub struct PhysicsBridgeSystem {
    physics_system: Arc<GpuPhysicsSystem>,
    particles: Vec<Particle>,
    frame_count: u32,
}

impl PhysicsBridgeSystem {
    /// Create a new physics bridge system
    pub fn new(physics_system: Arc<GpuPhysicsSystem>) -> Self {
        Self {
            physics_system,
            particles: Vec::new(),
            frame_count: 0,
        }
    }

    /// Configure the system
    pub fn config() -> super::SystemConfig {
        super::SystemConfig {
            stage: SystemStage::Early, // Run before other systems
            enabled: true,
            fixed_timestep: Some(1.0 / 60.0), // Physics runs at fixed timestep
        }
    }

    /// Synchronize physics data to GPU
    fn sync_to_gpu(&mut self) -> Result<(), PhysicsError> {
        self.physics_system.update_particles(&self.particles)?;
        Ok(())
    }

    /// Synchronize physics data from GPU
    fn sync_from_gpu(&mut self) -> Result<(), PhysicsError> {
        self.particles = self.physics_system.get_particle_data()?;
        Ok(())
    }
}

impl System for PhysicsBridgeSystem {
    fn update(&mut self, world: &mut World) {
        // Clear previous frame's data
        self.particles.clear();

        // Collect physics data from components
        for (entity, (transform, physics)) in
            world.query_mut::<(&mut TransformComponent, &mut PhysicsComponent)>()
        {
            if physics.enabled {
                // Update physics component with new data
                if let Some(particle_data) = physics.particle_data() {
                    self.particles.push(particle_data.clone());
                } else {
                    // Initialize particle data if not present
                    physics.update_particle_data().ok();
                    if let Some(particle_data) = physics.particle_data() {
                        self.particles.push(particle_data.clone());
                    }
                }
            }
        }

        // Sync with GPU physics system
        if let Err(e) = self.sync_to_gpu() {
            log::error!("Failed to sync physics data to GPU: {:?}", e);
            return;
        }

        // Run physics simulation
        self.physics_system.step();

        // Get updated physics data
        if let Err(e) = self.sync_from_gpu() {
            log::error!("Failed to sync physics data from GPU: {:?}", e);
            return;
        }

        // Update components with new physics state
        for (entity, (transform, physics)) in
            world.query_mut::<(&mut TransformComponent, &mut PhysicsComponent)>()
        {
            if physics.enabled {
                if let Some(particle) = self.particles.get(entity.index()) {
                    // Update transform
                    transform.set_position(particle.position);

                    // Update physics component
                    physics.position = particle.position;
                    physics.velocity = particle.velocity;
                    physics.acceleration = particle.acceleration;
                }
            }
        }

        self.frame_count = self.frame_count.wrapping_add(1);
    }

    fn initialize(&mut self, _world: &mut World) {
        // Initialize GPU physics resources
        log::info!("Initializing physics bridge system");
    }

    fn cleanup(&mut self, _world: &mut World) {
        // Clean up physics resources
        self.particles.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_bridge_config() {
        let config = PhysicsBridgeSystem::config();
        assert_eq!(config.stage, SystemStage::Early);
        assert!(config.enabled);
        assert_eq!(config.fixed_timestep, Some(1.0 / 60.0));
    }
}
