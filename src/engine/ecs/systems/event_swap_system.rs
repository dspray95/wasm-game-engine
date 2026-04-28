use crate::engine::{
    ecs::{ system::SystemContext, world::World },
    events::event_registry::{ EventRegistry, EventSwapFn },
};

pub fn event_swap_system(world: &mut World, _ctx: &mut SystemContext) {
    let fns: Vec<EventSwapFn> = world
        .get_resource::<EventRegistry>()
        .map(|registry| registry.swap_fns.clone())
        .unwrap_or_default();
    for swap_function in fns {
        swap_function(world);
    }
}
