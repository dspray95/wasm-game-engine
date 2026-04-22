use std::{ any::{ Any, TypeId }, collections::HashMap };

use cgmath::{ Deg, Vector3 };

use crate::engine::{
    ecs::{
        components::{
            camera::{
                camera::{ Camera, SurfaceDimensions },
                constants::{ DEFAULT_FAR, DEFAULT_FOV, DEFAULT_NEAR },
                projection::Projection,
            },
            transform::Transform,
        },
        entity::{ self, Entity, EntityAllocator },
        resources::camera::ActiveCamera,
        sparse_set::SparseSet,
    },
    state::context::GpuContext,
};

// Trait that lets World call remove() on a type-erased SparseSet without knowing T.
// as_any / as_any_mut allow downcasting back to SparseSet<T> when T is known.
trait ComponentStorage {
    fn remove(&mut self, entity_id: u32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn entity_ids(&self) -> Vec<u32>;
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
    fn entity_ids(&self) -> Vec<u32> {
        self.iter()
            .map(|(id, _)| id)
            .collect()
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
        self.components.entry(type_id).or_insert_with(|| Box::new(SparseSet::<T>::new()));
        let storage = self.components.get_mut(&type_id).unwrap();
        let set = storage.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.insert(entity.id, value);
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id)?;
        let set = storage.as_any().downcast_ref::<SparseSet<T>>().unwrap();
        set.get(entity.id)
    }

    pub fn get_component_by_id<T: 'static>(&self, entity_id: u32) -> Option<&T> {
        self.get_storage::<T>()?.get(entity_id)
    }

    pub fn get_component_mut_by_id<T: 'static>(&mut self, entity_id: u32) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get_mut(&type_id)?;
        let set = storage.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        set.get_mut(entity_id)
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

    pub fn spawn_entity_only(&mut self) -> Entity {
        self.entities.spawn()
    }

    pub fn spawn(&mut self) -> EntityBuilder {
        let entity = self.spawn_entity_only();
        EntityBuilder { world: self, entity }
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

    fn get_storage<T: 'static>(&self) -> Option<&SparseSet<T>> {
        self.components.get(&TypeId::of::<T>())?.as_any().downcast_ref()
    }

    pub(crate) fn entity_ids_for(&self, type_id: TypeId) -> Vec<u32> {
        self.components
            .get(&type_id)
            .map(|s| s.entity_ids())
            .unwrap_or_default()
    }

    // Use when we only need the entity's ID itself, if you want comoponents user query_iter
    pub fn iter_component<T: 'static>(&self) -> impl Iterator<Item = (u32, &T)> {
        self.get_storage::<T>()
            .map(|s| s.iter())
            .into_iter()
            .flatten()
    }

    pub fn get_entities_with<T: 'static>(&self) -> Vec<u32> {
        self.iter_component::<T>()
            .map(|(id, _)| id)
            .collect()
    }

    pub fn create_active_camera(&mut self, device: &wgpu::Device, position: Vector3<f32>) {
        let Some(camera_bind_group_layout) = self.get_resource::<wgpu::BindGroupLayout>() else {
            return;
        };
        let Some(surface_dimensions) = self.get_resource::<SurfaceDimensions>() else {
            return;
        };
        let (width, height) = (surface_dimensions.width as u32, surface_dimensions.height as u32);

        let projection = Projection::new(
            width,
            height,
            Deg(DEFAULT_FOV),
            DEFAULT_NEAR,
            DEFAULT_FAR
        );

        let render_pass_data = Camera::create_render_pass_data(device);

        let camera_component = Camera::new(
            Deg(90.0).into(),
            Deg(0.0).into(),
            projection,
            render_pass_data
        );

        let camera_entity = self
            .spawn()
            .with(Transform::new().with_position(position.x, position.y, position.z))
            .with(camera_component)
            .build();

        self.add_resource(ActiveCamera(camera_entity));
    }
}

pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityBuilder<'w> {
    pub fn with<T: 'static>(self, component: T) -> Self {
        self.world.add_component(self.entity, component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }
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
        let a = world.spawn_entity_only();
        let b = world.spawn_entity_only();
        assert_ne!(a, b);
    }

    #[test]
    fn despawned_entity_is_no_longer_alive_and_slot_is_recycled() {
        let mut world = World::new();
        let a = world.spawn_entity_only();
        assert!(world.is_alive(a));
        world.despawn(a);
        assert!(!world.is_alive(a));
        let b = world.spawn_entity_only();
        // new entity is alive, old stale handle is not
        assert!(world.is_alive(b));
        assert!(!world.is_alive(a));
    }

    // --- add / get component ---

    #[test]
    fn add_and_get_component() {
        let mut world = world_with_position();
        let e = world.spawn_entity_only();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    #[test]
    fn get_component_returns_none_when_not_added() {
        let mut world = world_with_position();
        let e = world.spawn_entity_only();
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn get_component_mut_allows_mutation() {
        let mut world = world_with_position();
        let e = world.spawn_entity_only();
        world.add_component(e, Position { x: 0.0, y: 0.0 });
        world.get_component_mut::<Position>(e).unwrap().x = 99.0;
        assert_eq!(world.get_component::<Position>(e).unwrap().x, 99.0);
    }

    // --- remove component ---

    #[test]
    fn removed_component_is_no_longer_present() {
        let mut world = world_with_position();
        let e = world.spawn_entity_only();
        world.add_component(e, Position { x: 1.0, y: 1.0 });
        world.remove_component::<Position>(e);
        assert!(world.get_component::<Position>(e).is_none());
    }

    #[test]
    fn remove_unregistered_component_does_not_panic() {
        let mut world = World::new();
        let e = world.spawn_entity_only();
        world.remove_component::<Position>(e); // no-op
    }

    // --- despawn cleans up components ---

    #[test]
    fn despawn_removes_all_components() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Health>();
        let e = world.spawn_entity_only();
        world.add_component(e, Position { x: 1.0, y: 1.0 });
        world.add_component(e, Health(100));
        world.despawn(e);
        assert!(world.get_component::<Position>(e).is_none());
        assert!(world.get_component::<Health>(e).is_none());
    }

    #[test]
    fn stale_handle_cannot_access_recycled_entity_components() {
        let mut world = world_with_position();
        let old = world.spawn_entity_only();
        world.add_component(old, Position { x: 5.0, y: 5.0 });
        world.despawn(old);
        let _new = world.spawn_entity_only(); // reuses same id slot
        // old handle's generation is stale — component was removed on despawn
        assert!(world.get_component::<Position>(old).is_none());
    }

    // --- EntityBuilder ---

    #[test]
    fn builder_attaches_all_components_to_same_entity() {
        let mut world = World::new();
        let e = world.spawn().with(Position { x: 1.0, y: 2.0 }).with(Health(42)).build();
        assert_eq!(world.get_component::<Position>(e).unwrap().x, 1.0);
        assert_eq!(world.get_component::<Health>(e).unwrap().0, 42);
    }

    #[test]
    fn builder_with_no_components_produces_live_entity() {
        let mut world = World::new();
        let e = world.spawn().build();
        assert!(world.is_alive(e));
    }

    #[test]
    fn two_builders_produce_distinct_entities() {
        let mut world = World::new();
        let a = world.spawn().with(Health(1)).build();
        let b = world.spawn().with(Health(2)).build();
        assert_ne!(a, b);
        assert_eq!(world.get_component::<Health>(a).unwrap().0, 1);
        assert_eq!(world.get_component::<Health>(b).unwrap().0, 2);
    }

    // --- multiple components on one entity ---

    #[test]
    fn multiple_component_types_on_one_entity() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Health>();
        let e = world.spawn_entity_only();
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

    // --- iter_component ---

    #[test]
    fn iter_component_yields_all_entities_with_component() {
        let mut world = World::new();
        world.register_component::<Health>();
        let a = world.spawn_entity_only();
        let b = world.spawn_entity_only();
        let c = world.spawn_entity_only();
        world.add_component(a, Health(10));
        world.add_component(b, Health(20));
        world.add_component(c, Health(30));

        let mut values: Vec<u32> = world
            .iter_component::<Health>()
            .map(|(_, h)| h.0)
            .collect();
        values.sort();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn iter_component_only_yields_entities_that_have_it() {
        let mut world = World::new();
        world.register_component::<Position>();
        world.register_component::<Health>();
        let a = world.spawn_entity_only();
        let b = world.spawn_entity_only();
        world.add_component(a, Position { x: 1.0, y: 0.0 });
        world.add_component(a, Health(100));
        world.add_component(b, Position { x: 2.0, y: 0.0 });
        // b has no Health

        let health_ids: Vec<u32> = world
            .iter_component::<Health>()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(health_ids, vec![a.id]);
    }

    #[test]
    fn iter_component_empty_for_unregistered_type() {
        let world = World::new();
        let count = world.iter_component::<Health>().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn iter_component_empty_after_all_despawned() {
        let mut world = World::new();
        world.register_component::<Health>();
        let e = world.spawn_entity_only();
        world.add_component(e, Health(50));
        world.despawn(e);
        assert_eq!(world.iter_component::<Health>().count(), 0);
    }
}
