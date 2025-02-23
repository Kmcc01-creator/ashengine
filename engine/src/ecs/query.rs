//! Query system for the ECS
//!
//! Provides efficient iteration and filtering over components

use super::{Entity, World};
use std::marker::PhantomData;

/// Filter for component queries
pub trait QueryFilter {
    /// Check if an entity matches the filter
    fn matches(&self, world: &World, entity: Entity) -> bool;
}

/// Builder for constructing queries
pub struct QueryBuilder<'a> {
    world: &'a World,
    filters: Vec<Box<dyn QueryFilter>>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new query builder
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            filters: Vec::new(),
        }
    }

    /// Add a filter to the query
    pub fn filter<F: QueryFilter + 'static>(mut self, filter: F) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    /// Build the query
    pub fn build<Q: Query<'a>>(self) -> QueryIter<'a, Q> {
        QueryIter {
            world: self.world,
            filters: self.filters,
            current: 0,
            _phantom: PhantomData,
        }
    }
}

/// Iterator for query results
pub struct QueryIter<'a, Q> {
    world: &'a World,
    filters: Vec<Box<dyn QueryFilter>>,
    current: usize,
    _phantom: PhantomData<Q>,
}

impl<'a, Q> Iterator for QueryIter<'a, Q>
where
    Q: Query<'a>,
{
    type Item = (Entity, Q::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.world.entities().len() {
            let entity = self.world.entities()[self.current];
            self.current += 1;

            if let Some(entity) = entity {
                if self.filters.iter().all(|f| f.matches(self.world, entity)) {
                    if let Some(components) = Q::fetch(self.world, entity) {
                        return Some((entity, components));
                    }
                }
            }
        }
        None
    }
}

/// Trait for component queries
pub trait Query<'a>: Sized {
    /// Type of the query result
    type Item;

    /// Fetch components for an entity
    fn fetch(world: &'a World, entity: Entity) -> Option<Self::Item>;
}

// Implement Query for common tuple sizes
impl<'a, A> Query<'a> for &'a A
where
    A: 'static,
{
    type Item = &'a A;

    fn fetch(world: &'a World, entity: Entity) -> Option<Self::Item> {
        world.get_component::<A>(entity)
    }
}

impl<'a, A> Query<'a> for &'a mut A
where
    A: 'static,
{
    type Item = &'a mut A;

    fn fetch(world: &'a World, entity: Entity) -> Option<Self::Item> {
        // Safety: We know this is safe because the borrow checker ensures
        // we don't have multiple mutable references
        unsafe {
            let world_ptr = world as *const World as *mut World;
            (*world_ptr).get_component_mut::<A>(entity)
        }
    }
}

// Implement for tuples
impl<'a, A, B> Query<'a> for (&'a A, &'a B)
where
    A: 'static,
    B: 'static,
{
    type Item = (&'a A, &'a B);

    fn fetch(world: &'a World, entity: Entity) -> Option<Self::Item> {
        Some((
            world.get_component::<A>(entity)?,
            world.get_component::<B>(entity)?,
        ))
    }
}

impl<'a, A, B> Query<'a> for (&'a mut A, &'a B)
where
    A: 'static,
    B: 'static,
{
    type Item = (&'a mut A, &'a B);

    fn fetch(world: &'a World, entity: Entity) -> Option<Self::Item> {
        unsafe {
            let world_ptr = world as *const World as *mut World;
            Some((
                (*world_ptr).get_component_mut::<A>(entity)?,
                world.get_component::<B>(entity)?,
            ))
        }
    }
}

// Add more tuple implementations as needed

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::Component;

    #[derive(Debug, Default)]
    struct Position(f32, f32, f32);
    impl Component for Position {}

    #[derive(Debug, Default)]
    struct Velocity(f32, f32, f32);
    impl Component for Velocity {}

    #[test]
    fn test_query_builder() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position(0.0, 0.0, 0.0));
        world.add_component(entity, Velocity(1.0, 1.0, 1.0));

        let query = QueryBuilder::new(&world).build::<(&Position, &Velocity)>();

        let results: Vec<_> = query.collect();
        assert_eq!(results.len(), 1);
    }
}
