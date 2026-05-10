use std::{any::TypeId, collections::HashMap};

use crate::engine::ecs::{
    commands::{
        component_buffer::{AnyComponentBuffer, ComponentBuffer},
        entity_commands::EntityCommands,
        event_buffer::{AnyEventBuffer, EventBuffer},
    },
    entity::{Entity, EntityAllocator},
    world::World,
};

pub struct Commands {
    despawns: Vec<Entity>,
    component_inserts: HashMap<TypeId, Box<dyn AnyComponentBuffer>>,
    component_removes: Vec<(Entity, fn(&mut World, Entity))>,
    event_buffers: HashMap<TypeId, Box<dyn AnyEventBuffer>>,
    resource_updates: Vec<Box<dyn FnOnce(&mut World)>>,
    custom: Vec<Box<dyn FnOnce(&mut World, &mut EntityAllocator)>>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            despawns: Vec::new(),
            component_inserts: HashMap::new(),
            component_removes: Vec::new(),
            event_buffers: HashMap::new(),
            resource_updates: Vec::new(),
            custom: Vec::new(),
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.despawns.push(entity);
    }

    pub fn send_event<T: 'static + Send + Sync>(&mut self, event: T) {
        let buffer = self
            .event_buffers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(EventBuffer::<T>::new()));

        buffer
            .as_any_mut()
            .downcast_mut::<EventBuffer<T>>()
            .unwrap()
            .pending
            .push(event);
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let buffer = self
            .component_inserts
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentBuffer::<T>::new()));

        buffer
            .as_any_mut()
            .downcast_mut::<ComponentBuffer<T>>()
            .unwrap()
            .pending
            .push((entity, component));
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        self.component_removes.push((entity, |world, e| {
            world.remove_component::<T>(e);
        }));
    }

    pub fn update_resource<T, F>(&mut self, mutator: F)
    where
        T: 'static,
        F: FnOnce(&mut T) + 'static,
    {
        self.resource_updates.push(Box::new(move |world| {
            if let Some(r) = world.get_resource_mut::<T>() {
                mutator(r);
            }
        }));
    }

    pub fn spawn<'a>(&'a mut self, allocator: &mut EntityAllocator) -> EntityCommands<'a> {
        let entity = allocator.spawn();
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub fn apply(&mut self, world: &mut World, allocator: &mut EntityAllocator) {
        for buf in self.component_inserts.values_mut() {
            buf.flush(world);
        }
        for (entity, remover) in self.component_removes.drain(..) {
            remover(world, entity);
        }
        for buf in self.event_buffers.values_mut() {
            buf.flush(world);
        }
        for update in self.resource_updates.drain(..) {
            update(world);
        }
        for entity in self.despawns.drain(..) {
            world.despawn(entity, allocator);
        }
        for cmd in self.custom.drain(..) {
            cmd(world, allocator);
        }
    }

    /// Reset between systems without dropping allocations.
    pub fn clear(&mut self) {
        self.despawns.clear();
        for buf in self.component_inserts.values_mut() {
            buf.clear();
        }
        self.component_removes.clear();
        for buf in self.event_buffers.values_mut() {
            buf.clear();
        }
        self.resource_updates.clear();
        self.custom.clear();
    }
}
