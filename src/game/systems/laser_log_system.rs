use crate::{
    engine::{ ecs::{ system::SystemContext, world::World }, events::events::Events },
    game::events::laser_fired_event::LaserFiredEvent,
};

pub fn laser_log_system(world: &mut World, _ctx: &mut SystemContext) {
    if let Some(events) = world.get_resource::<Events<LaserFiredEvent>>() {
        for event in events.read() {
            println!("Laser fired at {:?}", event.origin);
        }
    }
}
