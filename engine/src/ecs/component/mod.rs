//! Component system implementation
//!
//! Components are pure data containers that can be attached to entities.
//! They implement efficient storage and access patterns.

use std::any::TypeId;

/// Unique identifier for a component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(TypeId);

/// Trait for all component types
pub trait Component: 'static + Sized + Send + Sync {
    /// Get the component's unique identifier
    fn component_id() -> ComponentId {
        ComponentId(TypeId::of::<Self>())
    }
}

/// Storage container for components
pub trait ComponentStorage {
    /// The type of component stored
    type Component: Component;

    /// Insert a component for an entity
    fn insert(&mut self, entity: Entity, component: Self::Component);

    /// Remove a component for an entity
    fn remove(&mut self, entity: Entity) -> Option<Self::Component>;

    /// Get a reference to a component for an entity
    fn get(&self, entity: Entity) -> Option<&Self::Component>;

    /// Get a mutable reference to a component for an entity
    fn get_mut(&mut self, entity: Entity) -> Option<&mut Self::Component>;
}

// Re-export common components
mod physics;
mod render;
mod transform;

pub use physics::PhysicsComponent;
pub use render::RenderComponent;
pub use transform::TransformComponent;

use super::Entity;
