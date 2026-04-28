use crate::engine::{ ecs::world::World, events::events::Events };

pub type EventSwapFn = fn(&mut World);

pub struct EventRegistry {
    pub swap_fns: Vec<EventSwapFn>,
}

impl EventRegistry {
    pub fn new() -> Self {
        Self { swap_fns: Vec::new() }
    }

    pub fn register<T: 'static>(&mut self) {
        // fn pointers are Copy, so swap_fns.clone() is a cheap memcpy of the Vec.
        // With Box<dyn Fn> we'd need Rc or Arc wrappers and the borrow dance gets harder.
        fn swap_for<T: 'static>(world: &mut World) {
            if let Some(events) = world.get_resource_mut::<Events<T>>() {
                events.swap();
            }
        }
        self.swap_fns.push(swap_for::<T>);
    }
}
