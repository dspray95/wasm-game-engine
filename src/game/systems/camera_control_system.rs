use cgmath::{ InnerSpace, Rad, Vector3 };

use crate::{
    engine::{
        ecs::{
            components::{ camera::camera::Camera, transform::Transform },
            system::SystemContext,
            world::World,
        },
    },
    game::{canyon_runner_scene::FreeCameraEnabled, input::{actions::Action, world_ext::InputWorldExt}},
};

const CAMERA_SPEED: f32 = -0.2;

pub fn camera_control_system(world: &mut World, _system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    // Toggle free cam
    {
        let free_cam = world.get_resource_mut::<FreeCameraEnabled>().unwrap();
        if key_bindings.is_action_just_pressed(&Action::ToggleFreeCamera, &input) {
            free_cam.0 = !free_cam.0;
        }
    }

    let free_cam_enabled = world.get_resource::<FreeCameraEnabled>().unwrap().0;
    if !free_cam_enabled {
        return;
    }

    let active_camera_entity = world.active_camera();
    let Some(yaw) = world.get_component::<Camera>(active_camera_entity).map(|c| c.yaw) else {
        return;
    };

    let (yaw_sin, yaw_cos) = yaw.0.sin_cos();
    let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    let up = forward.cross(right);

    // Now safe to mutably borrow Transform and Camera separately
    if let Some(transform) = world.get_component_mut::<Transform>(active_camera_entity) {
        if key_bindings.is_action_pressed(&Action::MoveForwards, &input){
            transform.position -= forward * CAMERA_SPEED;
        }
        if key_bindings.is_action_pressed(&Action::MoveBackwards, &input) {
            transform.position += forward * CAMERA_SPEED;
        }
        if key_bindings.is_action_pressed(&Action::MoveLeft, &input)  {
            transform.position += right * CAMERA_SPEED;
        }
        if key_bindings.is_action_pressed(&Action::MoveRight, &input) {
            transform.position -= right * CAMERA_SPEED;
        }
        if key_bindings.is_action_pressed(&Action::MoveUp, &input) {
            transform.position += up * CAMERA_SPEED;
        }
        if key_bindings.is_action_pressed(&Action::MoveDown, &input) {
            transform.position -= up * CAMERA_SPEED;
        }
    }

    if let Some(camera) = world.get_component_mut::<Camera>(active_camera_entity) {
        if key_bindings.is_action_pressed(&Action::RotateLeft, &input) {
            camera.yaw -= Rad(0.025);
        }
        if key_bindings.is_action_pressed(&Action::RotateRight, &input) {
            camera.yaw += Rad(0.025);
        }
    }
}
