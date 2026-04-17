use crate::{
    engine::ecs::{
        components::{ transform::Transform, velocity::Velocity },
        system::SystemContext,
        world::World,
    },
    game::{ components::hover_state::HoverState, helpers::starfighter::animate_hover },
};

pub fn hover_system(world: &mut World, _system_context: &mut SystemContext) {
    for (transform, velocity, hover_state) in world.query_iter::<
        (&Transform, &mut Velocity, &mut HoverState)
    >() {
        animate_hover(transform, velocity, hover_state);
    }
}
