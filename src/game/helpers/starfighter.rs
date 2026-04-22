use crate::{
    engine::{
        ecs::components::{ transform::Transform, velocity::Velocity },
        model::{ loader::load_model_from_obj_bytes, model::Model },
        state::context::GpuContext,
    },
    game::{
        assets::{ STARFIGHTER_MODEL_MTL, STARFIGHTER_MODEL_OBJ },
        components::hover_state::{ HoverDirection, HoverState },
    },
};

const HOVER_SPEED: f32 = 0.2;

pub fn animate_hover(transform: &Transform, velocity: &mut Velocity, hover_state: &mut HoverState) {
    if transform.position.y > hover_state.upper_limit {
        hover_state.direction = HoverDirection::Down;
    } else if transform.position.y < hover_state.lower_limit {
        hover_state.direction = HoverDirection::Up;
    }

    if hover_state.direction == HoverDirection::Up {
        velocity.y += HOVER_SPEED;
    } else {
        velocity.y -= HOVER_SPEED;
    }
}

pub fn load_model(gpu_context: &GpuContext) -> Model {
    load_model_from_obj_bytes(
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_MODEL_MTL,
        gpu_context,
        None,
        1
    )
}
