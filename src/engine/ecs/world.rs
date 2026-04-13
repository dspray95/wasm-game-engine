use std::{ any::{ Any, TypeId }, collections::HashMap };

use crate::engine::ecs::{ entity::{ Entity, EntityAllocator }, sparse_set::SparseSet };

// Trait that lets World call remove() on a type-erased SparseSet without knowing T.
// as_any / as_any_mut allow downcasting back to SparseSet<T> when T is known.
trait ComponentStorage {
    fn remove(&mut self, entity_id: u32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentStorage for SparseSet<T> {
    fn remove(&mut self, entity_id: u32) {
        self.remove(entity_id);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct World {
    entities: EntityAllocator,
    // Keyed by TypeId so each SparseSet<T> is stored and retrieved by its component type.
    // Box<dyn ComponentStorage> erases the type while still exposing remove() for despawn.
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn default() -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::default(),
            components: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn register_component<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, Box::new(SparseSet::<T>::new()));
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, value: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get_mut(&type_id).expect("component type not registered");
        let set = storage.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.insert(entity.id, value);
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        let set = storage.as_any().downcast_ref::<SparseSet<T>>().unwrap();
        set.get(entity.id)
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get_mut(&type_id)?;
        let set = storage.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.get_mut(entity.id)
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        let Some(storage) = self.components.get_mut(&type_id) else {
            return;
        };
        let set = storage.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.remove(entity.id);
    }

    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(&entity)
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.entities.despawn(&entity);
        for storage in self.components.values_mut() {
            storage.remove(entity.id);
        }
    }

    pub fn add_resource<T: 'static>(&mut self, value: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position { x: f32, y: f32 }
    struct Health(u32);
    struct Speed(f32);

    fn world_with_position() -> World {
        let mut world = World::new();
        world.register_component::<Position>();
        world
    }

    // --- spawn / despawn ---

    #[test]
    fn spawned_entities_are_unique() {
        let mut world = World::new();
        let a = world.spawn();
        let b = world.spawn();
        assert_ne!(a, b);
    }

    #[test]
    fn despawned_entity_is_no_longer_alive_and_slot_is_recycled() {
        let mut world = World::new();
        let a = world.spawn();
        assert!(world.is_alive(a));
        world.despawn(a);
        assert!(!world.is_alive(a));
        let b = world.spawn();
        // new entity is alive, old stale handle is not
        assert!(world.is_alive(b));
        assert!(!world.is_alive(a));
    }

    // --- add / get component ---

    #[test]
    fn add_and_get_component() {
        let mut world = world_with_position();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn get_component_returns_none_when_not_added() {
        let mut world = world_with_position();
        let e = world.spawn();
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn get_component_mut_allows_mutation() {
        let mut world = world_with_position();
        let e = world.spawn();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        world.get_component_mut::<Position>(e).unwrap().x = 99.0;
        assert_eq!(world.get_component::<Position>(e).unwrap().x, 99.0);
    }

    // --- remove component ---

    #[test]
    fn removed_component_is_no_longer_present() {
        let mut world = world_with_position();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 1.0 });
        world.remove_component::<Position>(e);
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn remove_unregistered_component_does_not_panic() {
        let mut world = World::new();
        let e = world.spawn();
        world.remove_component::<Position>(e); // no-op
    }

    // --- despawn cleans up components ---

    #[test]
    fn despawn_removes_all_components() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Health>();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 1.0 });
        world.add_component(e, Health(100));
        world.despawn(e);
        assert!(world.get_component::<Position>(e).is_none());
        assert!(world.get_component::<Health>(e).is_none());
    }

    #[test]
    fn stale_handle_cannot_access_recycled_entity_components() {
        let mut world = world_with_position();
        let old = world.spawn();
        world.add_component(old, Position { x: 5.0, y: 5.0 });
        world.despawn(old);
        let _new = world.spawn(); // reuses same id slot
        // old handle's generation is stale — component was removed on despawn
        assert!(world.get_component::<Position>(old).is_none());
    }

    // --- multiple components on one entity ---

    #[test]
    fn multiple_component_types_on_one_entity() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Health>();
        let e = world.spawn();
        world.add_component(e, Position { x: 3.0, y: 4.0 });
        world.add_component(e, Health(50));
        assert_eq!(world.get_component::<Position>(e).unwrap().x, 3.0);
        assert_eq!(world.get_component::<Health>(e).unwrap().0, 50);
    }

    // --- resources ---

    #[test]
    fn add_and_get_resource() {
        let mut world = World::new();
        world.add_resource::<Speed>(Speed(10.0));
        assert_eq!(world.get_resource::<Speed>().unwrap().0, 10.0);
    }

    #[test]
    fn get_resource_mut_allows_mutation() {
        let mut world = World::new();
        world.add_resource(Speed(1.0));
        world.get_resource_mut::<Speed>().unwrap().0 = 5.0;
        assert_eq!(world.get_resource::<Speed>().unwrap().0, 5.0);
    }

    #[test]
    fn get_missing_resource_returns_none() {
        let world = World::new();
        assert!(world.get_resource::<Speed>().is_none());
    }
}
