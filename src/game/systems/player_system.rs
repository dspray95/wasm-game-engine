use cgmath::Vector3;

use crate::{
    engine::ecs::{
        components::{transform::Transform, velocity::Velocity},
        resources::camera::ActiveCamera,
        system::SystemContext,
        world::World,
    },
    game::{
        components::{dead::Dead, player::Player},
        helpers::player_speed::effective_player_z_speed,
        input::{actions::Action, world_ext::InputWorldExt},
        resources::player_speed_scaling::PlayerSpeedScaling,
    },
};

const X_MIN: f32 = 23.5;
const X_MAX: f32 = 25.5;

pub fn player_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    if let Some(dead) = world.get_component::<Dead>(player_entity) {
        let base_speed = world
            .get_resource::<PlayerSpeedScaling>()
            .map(|s| s.z_speed.base)
            .unwrap_or(0.0);
        let progress =
            (dead.ramp_time_remaining / dead.ramp_duration).clamp(0.0, 1.0);
        let effective_z_speed = base_speed + (dead.speed_at_death - base_speed) * progress;
        let new_remaining = (dead.ramp_time_remaining - system_context.delta_time).max(0.0);
        system_context
            .commands()
            .update_component::<Dead, _>(player_entity, move |d| {
                d.ramp_time_remaining = new_remaining;
            });

        let camera_entity = world.get_resource::<ActiveCamera>().map(|ac| ac.0);
        if let Some(entity) = camera_entity {
            let dt = system_context.delta_time;
            system_context
                .commands()
                .update_component::<Transform, _>(entity, move |t| {
                    t.position += Vector3::new(0.0, 0.0, 1.0) * effective_z_speed * dt;
                });
        }
        return;
    }

    let pause_pressed = key_bindings.is_action_just_pressed(&Action::Pause, &input);

    if pause_pressed {
        system_context
            .commands()
            .update_component::<Player, _>(player_entity, move |p| {
                p.move_player = !p.move_player;
            });
    }

    let Some((player, transform, velocity)) =
        world.query::<(&Player, &Transform, &Velocity)>(player_entity.id)
    else {
        return;
    };

    // Decide this frame's behaviour from current state.
    let effective_move = player.move_player ^ pause_pressed;
    if !effective_move {
        return;
    }

    // Reads the score-driven curve and applies any active speed-modifying
    // powerups (Hyperdrive). Falls back to the local Transform speed if the
    // scaling resource is missing.
    let effective_z_speed = if world.get_resource::<PlayerSpeedScaling>().is_some() {
        effective_player_z_speed(world)
    } else {
        player.z_movement_speed
    };

    // Move player
    let z_velocity = velocity.z + effective_z_speed;
    let mut x_velocity = velocity.x;

    let moving_left = key_bindings.is_action_pressed(&Action::MoveLeft, &input);
    let moving_right = key_bindings.is_action_pressed(&Action::MoveRight, &input);

    if moving_left && !moving_right && transform.position.x < X_MAX {
        x_velocity += player.strafe_speed;
    }
    if moving_right && !moving_left && transform.position.x > X_MIN {
        x_velocity -= player.strafe_speed;
    }

    system_context
        .commands()
        .update_component::<Velocity, _>(player_entity, move |v| {
            v.x += x_velocity;
            v.z += z_velocity;
        });

    // Move the camera. Routed through commands because the player/transform
    // refs above are still holding a shared borrow on world, so direct
    // mutation via get_component_mut would conflict.
    let camera_entity = world.get_resource::<ActiveCamera>().map(|ac| ac.0);
    if let Some(entity) = camera_entity {
        let z_speed = effective_z_speed;
        let dt = system_context.delta_time;
        system_context
            .commands()
            .update_component::<Transform, _>(entity, move |t| {
                t.position += Vector3::new(0.0, 0.0, 1.0) * z_speed * dt;
            });
    }
}
