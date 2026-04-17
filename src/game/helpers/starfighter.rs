use crate::{
    engine::{
        ecs::components::{ transform::Transform, velocity::Velocity },
        model::{
            descriptor::{ self, ModelDescriptor },
            loader::load_model_from_descriptor,
            material::Material,
            mesh::{ MeshData, calculate_normals },
            model::Model,
        },
        resources,
        state::context::GpuContext,
    },
    game::{
        assets::STARFIGHTER_MODEL_RON,
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
    let descriptor: ModelDescriptor = ron
        ::from_str(STARFIGHTER_MODEL_RON)
        .expect("Failed to parse starfighter.ron");
    load_model_from_descriptor(&descriptor, gpu_context)
}
