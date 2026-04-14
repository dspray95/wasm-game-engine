use cgmath::{ InnerSpace, Rad, Vector3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        camera::camera::Camera,
        ecs::{ resources::input_state::InputState, system::SystemContext, world::World },
    },
    game::canyon_runner_scene::FreeCameraEnabled,
};

const CAMERA_SPEED: f32 = -0.2;

// TODO: Camera actions layer, rather than directly reading the keycodes
pub fn camera_control_system(world: &mut World, ctx: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    {
        let free_cam = world.get_resource_mut::<FreeCameraEnabled>().unwrap();
        if input.just_pressed(KeyCode::KeyP) && input.is_pressed(KeyCode::ControlLeft) {
            free_cam.0 = !free_cam.0;
        }
    }

    let free_cam_enabled = world.get_resource::<FreeCameraEnabled>().unwrap().0;
    let camera = world.get_resource_mut::<Camera>().unwrap();

    if free_cam_enabled {
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();

        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
            camera.position -= forward * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
            camera.position += forward * CAMERA_SPEED;
        }

        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
            camera.position += right * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
            camera.position -= right * CAMERA_SPEED;
        }

        let up = forward.cross(right);
        if input.is_pressed(KeyCode::Space) {
            camera.position += up * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::ControlLeft) {
            camera.position -= up * CAMERA_SPEED;
        }

        if input.is_pressed(KeyCode::KeyQ) {
            camera.yaw -= Rad(0.025);
        }
        if input.is_pressed(KeyCode::KeyE) {
            camera.yaw += Rad(0.025);
        }
    }

    // Always sync CPU state → GPU buffer every frame. The camera may have been moved
    // by scene logic (move_player) even when free cam is off, so the buffer must stay current.
    camera.update_view_projeciton();
    camera.update_position();
    ctx.queue.unwrap().write_buffer(
        &camera.render_pass_data.buffer,
        0,
        bytemuck::cast_slice(&[camera.render_pass_data.uniform_buffer])
    );
}
