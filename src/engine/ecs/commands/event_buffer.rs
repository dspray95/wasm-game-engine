use std::any::Any;

use crate::engine::{ecs::world::World, events::events::Events};

pub trait AnyEventBuffer: Any {
    fn flush(&mut self, world: &mut World);
    fn clear(&mut self);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct EventBuffer<T> {
    pub pending: Vec<T>,
}

impl<T> EventBuffer<T> {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }
}

impl<T: 'static + Send + Sync> AnyEventBuffer for EventBuffer<T> {
    fn flush(&mut self, world: &mut World) {
        if let Some(events) = world.get_resource_mut::<Events<T>>() {
            for event in self.pending.drain(..) {
                events.send(event);
            }
        } else if !self.pending.is_empty() {
            log::error!(
                "Commands::send_event: Events<{}> not registered; {} event(s) dropped. Did you forget world.register_event?",
                std::any::type_name::<T>(),
                self.pending.len()
            );
            self.pending.clear();
        }
    }

    fn clear(&mut self) {
        self.pending.clear();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
