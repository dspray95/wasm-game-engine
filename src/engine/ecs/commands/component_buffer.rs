use std::any::Any;

use crate::engine::ecs::{entity::Entity, world::World};

pub trait AnyComponentBuffer: Any {
    fn flush(&mut self, world: &mut World);
    fn clear(&mut self);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentBuffer<T> {
    pub pending: Vec<(Entity, T)>,
}

impl<T> ComponentBuffer<T> {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }
}

impl<T: 'static> AnyComponentBuffer for ComponentBuffer<T> {
    fn flush(&mut self, world: &mut World) {
        for (entity, component) in self.pending.drain(..) {
            world.add_component(entity, component);
        }
    }

    fn clear(&mut self) {
        self.pending.clear();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
