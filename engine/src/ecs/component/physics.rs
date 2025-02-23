//! Physics component for entity simulation
//!
//! Provides physics properties and state for entities, bridging
//! between the ECS and the GPU physics system.

use super::Component;
use crate::physics::{Particle, PhysicsError};
use glam::{Vec3, Vec4};

/// Component for entity physics properties
#[derive(Debug)]
pub struct PhysicsComponent {
    /// Current position
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Current acceleration
    pub acceleration: Vec3,
    /// Mass of the entity
    pub mass: f32,
    /// Bounding box for collision (min_x, min_y, min_z, radius)
    pub bounding_box: Vec4,
    /// Whether physics simulation is enabled
    pub enabled: bool,
    /// Whether the entity is static (immovable)
    pub is_static: bool,
    /// GPU particle data for physics system
    particle_data: Option<Particle>,
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0,
            bounding_box: Vec4::new(0.0, 0.0, 0.0, 1.0),
            enabled: true,
            is_static: false,
            particle_data: None,
        }
    }
}

impl PhysicsComponent {
    /// Create a new physics component
    pub fn new(position: Vec3, mass: f32, radius: f32) -> Self {
        Self {
            position,
            mass,
            bounding_box: Vec4::new(0.0, 0.0, 0.0, radius),
            ..Default::default()
        }
    }

    /// Set the velocity
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set whether the entity is static
    pub fn with_static(mut self, is_static: bool) -> Self {
        self.is_static = is_static;
        self
    }

    /// Update the internal particle data for GPU physics
    pub fn update_particle_data(&mut self) -> Result<(), PhysicsError> {
        let particle = Particle {
            position: self.position,
            velocity: self.velocity,
            acceleration: self.acceleration,
            mass: self.mass,
            bounding_box: self.bounding_box,
        };
        self.particle_data = Some(particle);
        Ok(())
    }

    /// Get the current particle data for GPU physics
    pub fn particle_data(&self) -> Option<&Particle> {
        self.particle_data.as_ref()
    }

    /// Apply an impulse force
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_static && self.enabled {
            self.velocity += impulse / self.mass;
        }
    }

    /// Apply a continuous force
    pub fn apply_force(&mut self, force: Vec3) {
        if !self.is_static && self.enabled {
            self.acceleration += force / self.mass;
        }
    }
}

impl Component for PhysicsComponent {}

// Bridge implementation for physics system integration
impl PhysicsComponent {
    /// Convert to physics system format
    pub(crate) fn to_physics_data(&self) -> Option<Particle> {
        self.particle_data.clone()
    }
}
