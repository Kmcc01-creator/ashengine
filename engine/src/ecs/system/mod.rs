//! System implementations for the ECS
//!
//! Systems process entities and their components, implementing game logic
//! and handling interactions between different parts of the engine.
//!
//! The rendering system provides unified graphics integration, handling both
//! ECS data collection and graphics system interaction efficiently.

use super::World;
use std::any::TypeId;

/// Unique identifier for a system type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(TypeId);

/// Trait for implementing systems
pub trait System: 'static + Send + Sync {
    /// Get the system's unique identifier
    fn system_id(&self) -> SystemId {
        SystemId(TypeId::of::<Self>())
    }

    /// Update the system
    fn update(&mut self, world: &mut World);

    /// Optional initialization
    fn initialize(&mut self, _world: &mut World) {}

    /// Optional cleanup
    fn cleanup(&mut self, _world: &mut World) {}
}

// System implementations
mod physics_bridge;
mod render_system;

pub use physics_bridge::PhysicsBridgeSystem;
pub use render_system::RenderSystem;

/// System execution stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemStage {
    /// Early update (input, physics)
    Early,
    /// Main update (game logic)
    Update,
    /// Late update (rendering)
    Late,
}

/// Configuration for system execution
#[derive(Debug)]
pub struct SystemConfig {
    /// System execution stage
    pub stage: SystemStage,
    /// Whether the system is enabled
    pub enabled: bool,
    /// Optional fixed timestep for the system
    pub fixed_timestep: Option<f32>,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            stage: SystemStage::Update,
            enabled: true,
            fixed_timestep: None,
        }
    }
}

/// System scheduler for managing system execution
pub struct SystemScheduler {
    systems: Vec<(Box<dyn System>, SystemConfig)>,
}

impl SystemScheduler {
    /// Create a new system scheduler
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system with configuration
    pub fn add_system<S: System + 'static>(&mut self, system: S, config: SystemConfig) {
        self.systems.push((Box::new(system), config));
        // Sort systems by stage to ensure correct execution order
        self.systems.sort_by_key(|(_, config)| config.stage);
    }

    /// Update all systems
    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        for (system, config) in self.systems.iter_mut() {
            if config.enabled {
                match config.fixed_timestep {
                    Some(step) => {
                        // TODO: Implement fixed timestep logic
                        system.update(world);
                    }
                    None => system.update(world),
                }
            }
        }
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
