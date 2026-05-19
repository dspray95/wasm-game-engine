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

/// Deferred-mutation buffer for systems. Systems queue up entity / component /
/// resource / event mutations against `Commands` rather than mutating `World`
/// directly. The schedule applies the queued operations after each system
/// returns, which keeps `World` borrows short and avoids the need for systems
/// to fight the borrow checker over `&mut` access.
///
/// A `Commands` lives on `AppState` and is lent into each system via
/// `SystemContext`. Between systems the schedule calls `clear()` (resets
/// length, keeps capacity) and then `apply()` (drains queued ops into the
/// world).
///
/// **Visibility:** A system cannot see its own queued mutations during its
/// run — they only land after the system returns. Subsequent systems do see
/// them. If you need read-after-write within one system, mutate `World`
/// directly instead.
pub struct Commands {
    despawns: Vec<Entity>,
    component_inserts: HashMap<TypeId, Box<dyn AnyComponentBuffer>>,
    component_updates: Vec<Box<dyn FnOnce(&mut World)>>,
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
            component_updates: Vec::new(),
            component_removes: Vec::new(),
            event_buffers: HashMap::new(),
            resource_updates: Vec::new(),
            custom: Vec::new(),
        }
    }

    /// Queue an entity for despawn. The entity stays alive for the rest of
    /// the current system; subsequent systems see it as gone.
    ///
    /// # Example
    /// ```ignore
    /// for entity in dead_enemies {
    ///     system_context.commands().despawn(entity);
    /// }
    /// ```
    pub fn despawn(&mut self, entity: Entity) {
        self.despawns.push(entity);
    }

    /// Queue an event to be sent into the `Events<T>` resource at apply time.
    /// Events are pushed into a typed buffer per `T`, so this allocates only
    /// the first time an event of a given type is sent — subsequent sends
    /// reuse the existing buffer.
    ///
    /// The event type must be registered on the world via
    /// `world.register_event::<T>()` before any system sends it.
    ///
    /// # Example
    /// ```ignore
    /// system_context.commands().send_event(EnemyKilledEvent {
    ///     origin: transform.position,
    /// });
    /// ```
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

    /// Queue a component to be attached to an entity at apply time. Components
    /// are batched per type internally, so per-component allocation is avoided
    /// at scale.
    ///
    /// Prefer the builder form (`commands.spawn(...).with(...)`) when
    /// constructing a new entity. Use `add_component` for adding components
    /// to entities that already exist.
    ///
    /// # Example
    /// ```ignore
    /// system_context.commands().add_component(entity, Health(100));
    /// ```
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

    /// Queue a closure that mutates an entity's existing component of type
    /// `T` at apply time. Errors if the entity doesn't have the component.
    ///
    /// The turbofish on `T` is required so the queue knows which component
    /// type to fetch. The closure owns its captures (`move`) because it
    /// outlives the system body.
    ///
    /// # Example
    /// ```ignore
    /// system_context.commands().update_component::<Player, _>(player_entity, move |p| {
    ///     p.move_enabled = !p.move_enabled;
    /// });
    /// ```
    pub fn update_component<T, F>(&mut self, entity: Entity, mutator: F)
    where
        T: 'static,
        F: FnOnce(&mut T) + 'static,
    {
        self.component_updates.push(Box::new(move |world| {
            if let Some(component) = world.get_component_mut::<T>(entity) {
                mutator(component);
            } else {
                log::error!(
                    "Commands::update_component: entity {} has no {}; closure dropped",
                    entity.id,
                    std::any::type_name::<T>()
                );
            }
        }));
    }

    /// Queue a component to be removed from an entity at apply time. If the
    /// entity doesn't have the component, the remove is a no-op.
    ///
    /// # Example
    /// ```ignore
    /// system_context.commands().remove_component::<Stunned>(entity);
    /// ```
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        self.component_removes.push((entity, |world, e| {
            world.remove_component::<T>(e);
        }));
    }

    /// Queue a closure that mutates resource `T` at apply time. The closure
    /// owns its captures (`move` keyword on the caller side) because it
    /// outlives the system body.
    ///
    /// The turbofish on `T` is required so the queue knows which resource
    /// to fetch. If the resource doesn't exist at apply time, the closure
    /// is silently skipped.
    ///
    /// Use this when you'd otherwise need `get_resource_mut` mid-system but
    /// can't because of conflicting borrows on `World`.
    ///
    /// # Example
    /// ```ignore
    /// let dead = dead_entities.clone();
    /// system_context.commands().update_resource::<LaserManager, _>(move |manager| {
    ///     manager.alive_lasers.retain(|e| !dead.contains(e));
    /// });
    /// ```
    pub fn update_resource<T, F>(&mut self, mutator: F)
    where
        T: 'static,
        F: FnOnce(&mut T) + 'static,
    {
        self.resource_updates.push(Box::new(move |world| {
            if let Some(r) = world.get_resource_mut::<T>() {
                mutator(r);
            } else {
                log::error!(
                    "Commands::update_resource: resource {} not present in world; closure dropped. Did you forget world.add_resource?",
                    std::any::type_name::<T>()
                );
            }
        }));
    }

    /// Allocate a new entity ID synchronously and start building it. The
    /// `Entity` returned by `.build()` is real and usable immediately, even
    /// though its components only land at apply time.
    ///
    /// Takes `&mut EntityAllocator` because the ID is minted up-front, not
    /// deferred — usually pulled from `system_context.entity_allocator`.
    ///
    /// # Example
    /// ```ignore
    /// let entity = system_context.commands()
    ///     .spawn(system_context.entity_allocator)
    ///     .with(Transform::new())
    ///     .with(Renderable { model_id })
    ///     .build();
    /// ```
    /// Queue a `set_parent(child, parent)` operation. At flush time, calls
    /// `World::set_parent`, which handles old-parent removal, new-parent
    /// `Children` insertion, and cycle detection.
    ///
    /// Use this when a system needs to reparent or initially parent an
    /// entity that already exists. For spawn-time parenting prefer
    /// `commands.spawn(...).as_child_of(parent).build()` once that builder
    /// surface is added.
    pub fn set_parent(&mut self, child: Entity, parent: Entity) {
        self.component_updates.push(Box::new(move |world| {
            world.set_parent(child, parent);
        }));
    }

    pub fn spawn<'a>(&'a mut self, allocator: &mut EntityAllocator) -> EntityCommands<'a> {
        let entity = allocator.spawn();
        EntityCommands {
            entity,
            commands: self,
        }
    }

    /// Drain all queued operations into `world`. Called by the schedule after
    /// each system returns. The order is:
    ///
    /// 1. Component inserts (so subsequent updates can see freshly-added components)
    /// 2. Component updates (mutate existing components)
    /// 3. Component removes
    /// 4. Events
    /// 5. Resource updates
    /// 6. Despawns (so events fire while entities still exist in components)
    /// 7. Custom commands (escape hatch, applied last)
    pub fn apply(&mut self, world: &mut World, allocator: &mut EntityAllocator) {
        for buf in self.component_inserts.values_mut() {
            buf.flush(world);
        }
        for update in self.component_updates.drain(..) {
            update(world);
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

    /// Reset between systems without dropping allocations. Each per-type
    /// buffer's `Vec` has its length zeroed but keeps capacity, so after a
    /// few frames allocations stabilise at peak-frame size and stay there.
    pub fn clear(&mut self) {
        self.despawns.clear();
        for buf in self.component_inserts.values_mut() {
            buf.clear();
        }
        self.component_updates.clear();
        self.component_removes.clear();
        for buf in self.event_buffers.values_mut() {
            buf.clear();
        }
        self.resource_updates.clear();
        self.custom.clear();
    }
}
