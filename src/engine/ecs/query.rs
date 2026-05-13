// Query API
//
// Two parallel traits, following the stdlib `iter`/`iter_mut` convention:
//
// - `Fetch<'w>` is read-only — only impl'd for `&'w T`. Methods using it
//   (`query`, `query_iter`) take `&World`, so reads don't lock the world.
//
// - `FetchMut<'w>` covers both `&'w T` and `&'w mut T`. Methods using it
//   (`query_mut`, `query_iter_mut`) take `&mut World` because they need
//   to be able to return mutable references.
//
// The macro generates tuple implementations for both, up to 6 components.
//
// Use `query` / `query_iter` when you only need reads — other reads on
// `world` will continue to work in parallel. Use `query_mut` /
// `query_iter_mut` when any element in the tuple is `&mut T`.
//
// Safety invariant (FetchMut only): never include the same type twice in
// a tuple — that would produce two &mut references to the same data (UB).
// Different types always live in different SparseSets, so there is no
// aliasing across distinct types.

use std::any::TypeId;
use std::marker::PhantomData;

use super::world::World;

pub trait FetchMut<'w> {
    type Item;
    fn primary_type_id() -> TypeId;
    /// # Safety
    /// Caller must ensure no aliasing occurs (no duplicate types in a tuple query).
    unsafe fn fetch(world: *mut World, id: u32) -> Option<Self::Item>;
}

impl<'w, T: 'static> FetchMut<'w> for &'w T {
    type Item = &'w T;
    fn primary_type_id() -> TypeId {
        TypeId::of::<T>()
    }
    unsafe fn fetch(world: *mut World, id: u32) -> Option<Self::Item> {
        (*world).get_component_by_id::<T>(id)
    }
}

impl<'w, T: 'static> FetchMut<'w> for &'w mut T {
    type Item = &'w mut T;
    fn primary_type_id() -> TypeId {
        TypeId::of::<T>()
    }
    unsafe fn fetch(world: *mut World, id: u32) -> Option<Self::Item> {
        (*world).get_component_mut_by_id::<T>(id)
    }
}

// The first type in the tuple drives iteration — put the rarest component first
// for best performance (fewer entities to test against remaining components).
macro_rules! impl_fetch_tuple {
    ($first:ident $(, $rest:ident)*) => {
        impl<'w, $first: FetchMut<'w> $(, $rest: FetchMut<'w>)*> FetchMut<'w> for ($first, $($rest,)*) {
            type Item = ($first::Item, $($rest::Item,)*);
            fn primary_type_id() -> TypeId { $first::primary_type_id() }
            unsafe fn fetch(world: *mut World, id: u32) -> Option<Self::Item> {
                Some((
                    $first::fetch(world, id)?,
                    $($rest::fetch(world, id)?,)*
                ))
            }
        }
    }
}

impl_fetch_tuple!(A, B);
impl_fetch_tuple!(A, B, C);
impl_fetch_tuple!(A, B, C, D);
impl_fetch_tuple!(A, B, C, D, E);
impl_fetch_tuple!(A, B, C, D, E, F);

// --- QueryIterMut ---

pub struct QueryIterMut<'w, F: FetchMut<'w>> {
    world: *mut World,
    ids: std::vec::IntoIter<u32>,
    _phantom: PhantomData<fn() -> F::Item>,
}

impl<'w, F: FetchMut<'w>> Iterator for QueryIterMut<'w, F> {
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.ids.next()?;
            // Safety: same invariant as query_mut() — no duplicate types in F.
            // Entities missing any component in the tuple are skipped.
            unsafe {
                if let Some(item) = F::fetch(self.world, id) {
                    return Some(item);
                }
            }
        }
    }
}

// --- Read-only Fetch ---
//
// Mirror of `FetchMut` but only implemented for shared references `&T`.
// Because the returned items can't alias mutably, the trait is safe and
// the World access is `&World` rather than `&mut World` — meaning read-only
// queries don't lock the world.

pub trait Fetch<'w> {
    type Item;
    fn primary_type_id() -> TypeId;
    fn fetch(world: &'w World, id: u32) -> Option<Self::Item>;
}

impl<'w, T: 'static> Fetch<'w> for &'w T {
    type Item = &'w T;
    fn primary_type_id() -> TypeId { TypeId::of::<T>() }
    fn fetch(world: &'w World, id: u32) -> Option<Self::Item> {
        world.get_component_by_id::<T>(id)
    }
}

macro_rules! impl_fetch_tuple {
    ($first:ident $(, $rest:ident)*) => {
        impl<'w, $first: Fetch<'w> $(, $rest: Fetch<'w>)*> Fetch<'w> for ($first, $($rest,)*) {
            type Item = ($first::Item, $($rest::Item,)*);
            fn primary_type_id() -> TypeId { $first::primary_type_id() }
            fn fetch(world: &'w World, id: u32) -> Option<Self::Item> {
                Some((
                    $first::fetch(world, id)?,
                    $($rest::fetch(world, id)?,)*
                ))
            }
        }
    }
}

impl_fetch_tuple!(A, B);
impl_fetch_tuple!(A, B, C);
impl_fetch_tuple!(A, B, C, D);
impl_fetch_tuple!(A, B, C, D, E);
impl_fetch_tuple!(A, B, C, D, E, F);

pub struct QueryIter<'w, F: Fetch<'w>> {
    world: &'w World,
    ids: std::vec::IntoIter<u32>,
    _phantom: PhantomData<fn() -> F::Item>,
}

impl<'w, F: Fetch<'w>> Iterator for QueryIter<'w, F> {
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.ids.next()?;
            if let Some(item) = F::fetch(self.world, id) {
                return Some(item);
            }
        }
    }
}

// --- World methods ---

impl World {
    /// Read-only query. Takes `&self`, so other reads on `world` remain
    /// available while the result is alive. Tuple elements must be `&T`
    /// (not `&mut T`) — use `query_mut` if you need mutation.
    pub fn query<'w, F: Fetch<'w>>(&'w self, id: u32) -> Option<F::Item> {
        F::fetch(self, id)
    }

    /// Read-only counterpart to `query_iter_mut`. Iterates all entities that
    /// have the primary (first) component in `F`, yielding only those that
    /// also have every other component in the tuple.
    pub fn query_iter<'w, F: Fetch<'w>>(&'w self) -> QueryIter<'w, F> {
        let ids = self.entity_ids_for(F::primary_type_id());
        QueryIter {
            world: self,
            ids: ids.into_iter(),
            _phantom: PhantomData,
        }
    }

    /// Mutable query. Use when the tuple contains any `&mut T`. Locks
    /// `World` mutably for as long as the returned references live —
    /// other reads on `world` are blocked until the result is dropped.
    pub fn query_mut<'w, F: FetchMut<'w>>(&'w mut self, id: u32) -> Option<F::Item> {
        unsafe { F::fetch(self as *mut World, id) }
    }

    /// Mutable iterator variant.
    pub fn query_iter_mut<'w, F: FetchMut<'w>>(&'w mut self) -> QueryIterMut<'w, F> {
        let ids = self.entity_ids_for(F::primary_type_id());
        QueryIterMut {
            world: self as *mut World,
            ids: ids.into_iter(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Health(u32);

    // --- query (single entity) ---

    #[test]
    fn immutable_fetch_returns_component() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world
            .spawn(&mut alloc)
            .with(Position { x: 1.0, y: 2.0 })
            .build();
        let pos = world.query::<&Position>(e.id).unwrap();
        assert_eq!(pos.x, 1.0);
    }

    #[test]
    fn mutable_fetch_allows_mutation() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world
            .spawn(&mut alloc)
            .with(Position { x: 0.0, y: 0.0 })
            .build();
        world.query_mut::<&mut Position>(e.id).unwrap().x = 42.0;
        assert_eq!(world.query::<&Position>(e.id).unwrap().x, 42.0);
    }

    #[test]
    fn fetch_returns_none_for_missing_component() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world.spawn(&mut alloc).build();
        assert!(world.query::<&Position>(e.id).is_none());
    }

    #[test]
    fn fetch_returns_none_for_unknown_entity_id() {
        let mut world = World::new();
        assert!(world.query::<&Position>(999).is_none());
    }

    #[test]
    fn two_component_mutable_query_mutates_both() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world
            .spawn(&mut alloc)
            .with(Position { x: 0.0, y: 0.0 })
            .with(Velocity { x: 0.0, y: 0.0 })
            .build();
        {
            let (pos, vel) = world.query_mut::<(&mut Position, &mut Velocity)>(e.id).unwrap();
            pos.x = 10.0;
            vel.x = 5.0;
        }
        assert_eq!(world.query::<&Position>(e.id).unwrap().x, 10.0);
        assert_eq!(world.query::<&Velocity>(e.id).unwrap().x, 5.0);
    }

    #[test]
    fn tuple_query_returns_none_if_any_component_missing() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world
            .spawn(&mut alloc)
            .with(Position { x: 1.0, y: 0.0 })
            .build();
        assert!(world.query::<(&Position, &Velocity)>(e.id).is_none());
    }

    #[test]
    fn three_component_query() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        let e = world
            .spawn(&mut alloc)
            .with(Position { x: 0.0, y: 0.0 })
            .with(Velocity { x: 0.0, y: 0.0 })
            .with(Health(100))
            .build();
        let (pos, vel, hp) = world
            .query_mut::<(&mut Position, &mut Velocity, &mut Health)>(e.id)
            .unwrap();
        pos.x = 1.0;
        vel.y = 2.0;
        hp.0 -= 10;
        assert_eq!(world.query::<&Position>(e.id).unwrap().x, 1.0);
        assert_eq!(world.query::<&Velocity>(e.id).unwrap().y, 2.0);
        assert_eq!(world.query::<&Health>(e.id).unwrap().0, 90);
    }

    // --- query_iter ---

    #[test]
    fn query_iter_yields_all_entities_with_component() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        for i in 0..3 {
            world
                .spawn(&mut alloc)
                .with(Position {
                    x: i as f32,
                    y: 0.0,
                })
                .build();
        }
        let xs: Vec<f32> = world.query_iter::<&Position>().map(|p| p.x).collect();
        assert_eq!(xs.len(), 3);
    }

    #[test]
    fn query_iter_skips_entities_missing_secondary_component() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        // Entity with both
        world
            .spawn(&mut alloc)
            .with(Position { x: 1.0, y: 0.0 })
            .with(Velocity { x: 0.0, y: 0.0 })
            .build();
        // Entity with only Position — should be skipped
        world
            .spawn(&mut alloc)
            .with(Position { x: 2.0, y: 0.0 })
            .build();

        let count = world.query_iter::<(&Position, &Velocity)>().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn query_iter_empty_when_no_entities_have_primary_component() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        world
            .spawn(&mut alloc)
            .with(Velocity { x: 1.0, y: 0.0 })
            .build();
        // No Position components — iterator driven by Position should be empty
        let count = world.query_iter::<(&Position, &Velocity)>().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn query_iter_mutation_affects_stored_components() {
        let mut world = World::new();
        let mut alloc = crate::engine::ecs::entity::EntityAllocator::default();
        for _ in 0..3 {
            world
                .spawn(&mut alloc)
                .with(Position { x: 0.0, y: 0.0 })
                .with(Velocity { x: 1.0, y: 0.0 })
                .build();
        }
        for (pos, vel) in world.query_iter_mut::<(&mut Position, &Velocity)>() {
            pos.x += vel.x;
        }
        let all_moved = world.query_iter::<&Position>().all(|p| p.x == 1.0);
        assert!(all_moved);
    }

    #[test]
    fn query_iter_empty_world_produces_no_items() {
        let mut world = World::new();
        assert_eq!(world.query_iter::<&Position>().count(), 0);
    }
}
