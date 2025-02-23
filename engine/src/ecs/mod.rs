//! Entity Component System (ECS) implementation for AshEngine
//!
//! This module provides a high-performance, cache-friendly ECS architecture
//! with bridge systems for compatibility with existing engine modules.

mod component;
mod query;
mod resource;
mod system;
mod world;

pub use component::{Component, ComponentId, ComponentStorage};
pub use query::{Query, QueryBuilder, QueryFilter};
pub use resource::{Resource, ResourceId, Resources};
pub use system::{System, SystemId};
pub use world::{Entity, EntityBuilder, World};

pub mod prelude {
    //! Commonly used types and traits

    pub use super::{
        Component, ComponentId, Entity, EntityBuilder, Query, QueryBuilder, Resource, System, World,
    };
}
