use cgmath::{ InnerSpace, Vector3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        camera::camera::Camera,
        ecs::{
            components::{ transform::Transform, velocity::{ self, Velocity } },
            resources::input_state::InputState,
            system::SystemContext,
            world::World,
        },
    },
    game::components::player::{ self, Player },
};

const Z_MOVEMENT_SPEED: f32 = 10.0;
const STRAFE_SPEED: f32 = 4.0;

pub fn player_system(world: &mut World, _system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    if let Some((velocity, _player)) = world.query_iter::<(&mut Velocity, &Player)>().next() {
        // Move forward
        velocity.z = Z_MOVEMENT_SPEED;

        // Strafe controls
        if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
            velocity.x = STRAFE_SPEED;
        }
        if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
            velocity.x = STRAFE_SPEED;
        }
    }

    if let Some(camera) = world.get_resource_mut::<Camera>() {
        // Also move the camera forward
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        camera.position += forward * Z_MOVEMENT_SPEED * _system_context.delta_time;
    }
}
