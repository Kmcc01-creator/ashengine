//! World implementation for the ECS
//!
//! The World struct is the main container for the ECS, managing entities,
//! components, and providing query functionality.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::component::{Component, ComponentId};
use super::system::{System, SystemScheduler};

/// Entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    id: usize,
    generation: usize,
    index: usize,
}

impl Entity {
    /// Get the entity's index
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Storage for a single component type
struct ComponentStorage {
    data: Box<dyn Any>,
    removed: Vec<usize>,
}

/// World containing all entities and components
pub struct World {
    entities: Vec<Option<Entity>>,
    components: HashMap<ComponentId, ComponentStorage>,
    next_entity_id: AtomicUsize,
    scheduler: SystemScheduler,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
            next_entity_id: AtomicUsize::new(0),
            scheduler: SystemScheduler::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let index = if let Some(reused_index) = self.find_free_index() {
            reused_index
        } else {
            self.entities.len()
        };

        let entity = Entity {
            id,
            generation: 0,
            index,
        };

        if index == self.entities.len() {
            self.entities.push(Some(entity));
        } else {
            self.entities[index] = Some(entity);
        }

        entity
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let component_id = T::component_id();

        let storage = self
            .components
            .entry(component_id)
            .or_insert_with(|| ComponentStorage {
                data: Box::new(Vec::<T>::new()),
                removed: Vec::new(),
            });

        let components = storage.data.downcast_mut::<Vec<T>>().unwrap();

        if entity.index >= components.len() {
            components.resize_with(entity.index + 1, Default::default);
        }
        components[entity.index] = component;
    }

    /// Get a reference to a component
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let storage = self.components.get(&T::component_id())?;
        let components = storage.data.downcast_ref::<Vec<T>>().unwrap();
        components.get(entity.index)
    }

    /// Get a mutable reference to a component
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let storage = self.components.get_mut(&T::component_id())?;
        let components = storage.data.downcast_mut::<Vec<T>>().unwrap();
        components.get_mut(entity.index)
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let storage = self.components.get_mut(&T::component_id())?;
        let components = storage.data.downcast_mut::<Vec<T>>().unwrap();
        if entity.index < components.len() {
            storage.removed.push(entity.index);
            Some(std::mem::take(&mut components[entity.index]))
        } else {
            None
        }
    }

    /// Delete an entity and all its components
    pub fn delete_entity(&mut self, entity: Entity) {
        if let Some(stored_entity) = self.entities.get_mut(entity.index) {
            if let Some(e) = stored_entity {
                if e.id == entity.id {
                    *stored_entity = None;
                    // Remove components
                    for storage in self.components.values_mut() {
                        storage.removed.push(entity.index);
                    }
                }
            }
        }
    }

    /// Add a system to the world
    pub fn add_system<S: System + 'static>(
        &mut self,
        system: S,
        config: super::system::SystemConfig,
    ) {
        self.scheduler.add_system(system, config);
    }

    /// Update all systems
    pub fn update(&mut self, delta_time: f32) {
        self.scheduler.update(self, delta_time);
    }

    /// Query for components
    pub fn query<'a, Q: Query<'a>>(&'a mut self) -> QueryIter<'a, Q> {
        Q::create_query(self)
    }

    // Helper methods
    fn find_free_index(&self) -> Option<usize> {
        self.entities.iter().position(|e| e.is_none())
    }
}

/// Trait for component queries
pub trait Query<'a> {
    type Item;

    fn create_query(world: &'a mut World) -> QueryIter<'a, Self>
    where
        Self: Sized;
}

/// Iterator for query results
pub struct QueryIter<'a, Q: Query<'a>> {
    world: &'a mut World,
    current: usize,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'a, Q: Query<'a>> Iterator for QueryIter<'a, Q> {
    type Item = Q::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.world.entities.len() {
            let index = self.current;
            self.current += 1;

            if let Some(entity) = self.world.entities[index] {
                // Query implementation would check for components here
                // and return the requested tuple
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.generation, 0);
        assert_eq!(entity.index, 0);
    }
}
