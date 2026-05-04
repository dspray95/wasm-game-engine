use crate::{
    engine::ecs::{
        components::{transform::Transform, velocity::Velocity},
        system::SystemContext,
        world::World,
    },
    game::components::hover_state::{HoverDirection, HoverState},
};

const HOVER_SPEED: f32 = 0.2;

pub fn hover_system(world: &mut World, _system_context: &mut SystemContext) {
    for (transform, velocity, hover_state) in
        world.query_iter::<(&Transform, &mut Velocity, &mut HoverState)>()
    {
        animate_hover(transform, velocity, hover_state);
    }
}

pub fn animate_hover(transform: &Transform, velocity: &mut Velocity, hover_state: &mut HoverState) {
    if transform.position.y > hover_state.upper_limit {
        hover_state.direction = HoverDirection::Down;
    } else if transform.position.y < hover_state.lower_limit {
        hover_state.direction = HoverDirection::Up;
    }

    match hover_state.direction {
        HoverDirection::Up => {
            velocity.y += HOVER_SPEED;
        }
        HoverDirection::Down => {
            velocity.y -= HOVER_SPEED;
        }
    }
}
