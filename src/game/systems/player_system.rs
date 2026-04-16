use cgmath::{ InnerSpace, Vector3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        camera::camera::Camera,
        ecs::{
            components::{ transform::Transform, velocity::Velocity },
            resources::input_state::InputState,
            system::SystemContext,
            world::World,
        },
    },
    game::components::player::Player,
};

const Z_MOVEMENT_SPEED: f32 = 10.0;
const STRAFE_SPEED: f32 = 4.0;

const X_MIN: f32 = 23.5;
const X_MAX: f32 = 25.5;

pub fn player_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    if
        let Some((_player, transform, velocity)) = world
            .query_iter::<(&Player, &Transform, &mut Velocity)>()
            .next()
    {
        velocity.z += Z_MOVEMENT_SPEED;

        if
            (input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft)) &&
            transform.position.x < X_MAX
        {
            velocity.x += STRAFE_SPEED;
        }
        if
            (input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight)) &&
            transform.position.x > X_MIN
        {
            velocity.x -= STRAFE_SPEED;
        }
    }

    if let Some(camera) = world.get_resource_mut::<Camera>() {
        // The camera isn't an entity, so velocity_system won't move it for us
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        camera.position += forward * Z_MOVEMENT_SPEED * system_context.delta_time;
    }
}
