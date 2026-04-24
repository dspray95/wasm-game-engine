use cgmath::{ Vector3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        ecs::{
            components::{ transform::Transform, velocity::Velocity },
            resources::camera::ActiveCamera,
            system::SystemContext,
            world::World,
        },
        input::input_state::InputState,
    },
    game::{components::player::Player, input::{actions::Action, bindings::Bindings}, resources::move_player::{self, MovePlayer}},
};

const Z_MOVEMENT_SPEED: f32 = 10.0;
const STRAFE_SPEED: f32 = 4.0;

const X_MIN: f32 = 23.5;
const X_MAX: f32 = 25.5;

pub fn player_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();
    let key_bindings = world.get_resource::<Bindings<Action>>().unwrap().clone();
    
    let move_player = {
        let resource = world.get_resource_mut::<MovePlayer>().unwrap();
        if key_bindings.is_action_just_pressed(&Action::Pause, &input) {
            resource.0 = !resource.0;
        }
        resource.0
    };


    if !move_player {
        return;
    }

    if
        let Some((_player, transform, velocity)) = world
            .query_iter::<(&Player, &Transform, &mut Velocity)>()
            .next()
    {
        velocity.z += Z_MOVEMENT_SPEED;

        let moving_left = key_bindings.is_action_pressed(&Action::MoveLeft, &input);
        let moving_right = key_bindings.is_action_pressed(&Action::MoveRight, &input);

        if moving_left && !moving_right && transform.position.x < X_MAX {
            velocity.x += STRAFE_SPEED;
        }
        if moving_right && !moving_left && transform.position.x > X_MIN {
            velocity.x -= STRAFE_SPEED;
        }
    }

    let camera_entity = world.get_resource::<ActiveCamera>().map(|ac| ac.0);
    if let Some(entity) = camera_entity {
        if let Some(camera_transform) = world.get_component_mut::<Transform>(entity) {
            let forward = Vector3::new(0.0, 0.0, 1.0);
            camera_transform.position += forward * Z_MOVEMENT_SPEED * system_context.delta_time;
        }
    }
}
