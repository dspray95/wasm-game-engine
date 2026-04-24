use cgmath::{ InnerSpace, Rad, Vector3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        ecs::{
            components::{ camera::camera::Camera, transform::Transform },
            resources::camera::ActiveCamera,
            system::SystemContext,
            world::World,
        },
        input::input_state::InputState,
    },
    game::canyon_runner_scene::FreeCameraEnabled,
};

const CAMERA_SPEED: f32 = -0.2;

// TODO: Camera actions layer, rather than directly reading the keycodes
pub fn camera_control_system(world: &mut World, _system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    // Toggle free cam
    {
        let free_cam = world.get_resource_mut::<FreeCameraEnabled>().unwrap();
        if input.just_pressed(KeyCode::KeyP) && input.is_pressed(KeyCode::ControlLeft) {
            free_cam.0 = !free_cam.0;
        }
    }

    let free_cam_enabled = world.get_resource::<FreeCameraEnabled>().unwrap().0;
    if !free_cam_enabled {
        return;
    }

    let Some(active_camera_entity) = world.get_resource::<ActiveCamera>().map(|ac| ac.0) else {
        return;
    };
    let Some(yaw) = world.get_component::<Camera>(active_camera_entity).map(|c| c.yaw) else {
        return;
    };

    let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
    let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    let up = forward.cross(right);

    // Now safe to mutably borrow Transform and Camera separately
    if let Some(transform) = world.get_component_mut::<Transform>(active_camera_entity) {
        if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
            transform.position -= forward * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
            transform.position += forward * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
            transform.position += right * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
            transform.position -= right * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::Space) {
            transform.position += up * CAMERA_SPEED;
        }
        if input.is_pressed(KeyCode::ControlLeft) {
            transform.position -= up * CAMERA_SPEED;
        }
    }

    if let Some(camera) = world.get_component_mut::<Camera>(active_camera_entity) {
        if input.is_pressed(KeyCode::KeyQ) {
            camera.yaw -= Rad(0.025);
        }
        if input.is_pressed(KeyCode::KeyE) {
            camera.yaw += Rad(0.025);
        }
    }
}
