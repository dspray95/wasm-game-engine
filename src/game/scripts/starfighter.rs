use crate::{
    engine::ecs::components::{ transform::Transform, velocity::Velocity },
    game::components::hover_state::{ HoverDirection, HoverState },
};

const HOVER_SPEED: f32 = 0.2;

pub fn animate_hover(
    transform: &Transform,
    current_velocity: &mut Velocity,
    hover_state: &mut HoverState
) {
    if transform.position.y > hover_state.upper_limit {
        hover_state.direction = HoverDirection::Down;
    } else if transform.position.y < hover_state.lower_limit {
        hover_state.direction = HoverDirection::Up;
    }

    if hover_state.direction == HoverDirection::Up {
        current_velocity.y = HOVER_SPEED;
    } else {
        current_velocity.y = -HOVER_SPEED;
    }
}
