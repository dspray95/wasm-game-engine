use crate::engine::{
    ecs::{ events::collision_event::CollisionEvent, system::SystemContext, world::World },
    events::events::Events,
};

pub fn collision_log_system(world: &mut World, _: &mut SystemContext) {
    if let Some(events) = world.get_resource::<Events<CollisionEvent>>() {
        for event in events.read() {
            log::info!(
                "collision: {:?} ↔ {:?} normal={:?} depth={}",
                event.a,
                event.b,
                event.normal,
                event.depth
            );
        }
    }
}
