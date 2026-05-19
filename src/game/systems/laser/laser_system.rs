use std::collections::HashSet;

use cgmath::{One, Quaternion, Vector3};
use web_time::Instant;

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            components::{
                renderable::Renderable, transform::Transform, velocity::Velocity,
                world_transform::WorldTransform,
            },
            entity::{Entity, EntityAllocator},
            system::SystemContext,
            world::World,
        },
    },
    game::{
        components::{
            dead::Dead,
            double_fire_rate::DoubleFireRate,
            glitch_vfx::GlitchVfx,
            invulnerable::Invulnerable,
            laser::{Laser, DEFAULT_TRAVEL_SPEED},
            player::Player,
        },
        helpers::player_speed::effective_player_z_speed,
        input::{actions::Action, world_ext::InputWorldExt},
        resources::{laser_resources::LaserManager, player_score::PlayerScore},
    },
};

pub fn laser_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    let player_position = world
        .query_iter::<(&Player, &WorldTransform)>()
        .next()
        .map(|(_, transform)| transform.position);

    let player_can_fire = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| {
            world.get_component_by_id::<Dead>(id).is_none()
                && world.get_component_by_id::<Invulnerable>(id).is_none()
        })
        .unwrap_or(false);

    let tried_to_fire = player_can_fire && key_bindings.is_action_pressed(&Action::Fire, &input);
    if tried_to_fire {
        let now = Instant::now();

        let player_z_speed = effective_player_z_speed(world);
        let player_score = world
            .get_resource::<PlayerScore>()
            .map(|s| s.score)
            .unwrap_or(0);
        let double_fire_rate = world
            .iter_component::<Player>()
            .next()
            .map(|(id, _)| world.get_component_by_id::<DoubleFireRate>(id).is_some())
            .unwrap_or(false);
        let is_allowed_to_fire = {
            let Some(laser_manager) = world.get_resource::<LaserManager>() else {
                return;
            };
            let mut cooldown_seconds = laser_manager.fire_cooldown.value(player_score);
            if double_fire_rate {
                cooldown_seconds *= 0.5;
            }
            laser_manager.is_allowed_to_fire(now, cooldown_seconds)
        };
        let laser_entity: Option<Entity> = if is_allowed_to_fire {
            Some(spawn_laser(
                world,
                system_context.asset_server.as_deref().unwrap(),
                player_position.unwrap(),
                Vector3 {
                    x: 10.0,
                    y: 10.0,
                    z: 10.0,
                },
                now,
                player_z_speed,
                system_context.entity_allocator,
            ))
        } else {
            None
        };

        // New laser beam created
        if let Some(laser) = laser_entity {
            let Some(laser_manager) = world.get_resource_mut::<LaserManager>() else {
                return;
            };
            laser_manager.alive_lasers.push(laser);
            laser_manager.last_fired_time = now;

            // Attach glitch siblings if DoubleFireRate is active. They parent
            // to the laser so they move with it and despawn-cascade with it.
            if double_fire_rate {
                spawn_laser_glitch_pair(
                    world,
                    system_context.asset_server.as_deref().unwrap(),
                    laser,
                    system_context.entity_allocator,
                );
            }
        }
    }

    let (alive_lasers, max_travel_distance) = {
        let laser_manager = world.get_resource::<LaserManager>().unwrap();
        (
            laser_manager.alive_lasers.clone(),
            laser_manager.max_travel_distance,
        )
    };

    // Move/despawn beams
    let mut to_despawn: Vec<Entity> = Vec::new();

    for laser_entity in alive_lasers {
        if let Some((laser, transform, velocity)) =
            world.query_mut::<(&mut Laser, &mut Transform, &mut Velocity)>(laser_entity.id)
        {
            let distance_travelled = (transform.position.z - laser.initial_z).abs();
            if distance_travelled > max_travel_distance {
                to_despawn.push(laser_entity);
            } else {
                velocity.z = laser.travel_speed;
            }
        }
    }

    let despawn_ids: HashSet<u32> = to_despawn.iter().map(|entity| entity.id).collect();
    let laser_manager = world.get_resource_mut::<LaserManager>().unwrap();
    laser_manager
        .alive_lasers
        .retain(|entity| !despawn_ids.contains(&entity.id));

    for entity in to_despawn {
        system_context.commands.despawn(entity);
    }
}

// Local-space offset; gets multiplied by the laser's scale (~10) during
// hierarchy composition, so 0.005 here ≈ 0.05 world units of shoogle.
const LASER_GLITCH_MAX_OFFSET: f32 = 0.001;

fn spawn_laser_glitch_pair(
    world: &mut World,
    asset_server: &AssetServer,
    laser_entity: Entity,
    allocator: &mut EntityAllocator,
) {
    let cyan_id = asset_server.get_model_id("laser_glitch_cyan");
    let white_id = asset_server.get_model_id("laser_glitch_white");
    spawn_laser_glitch_child(world, cyan_id, laser_entity, allocator);
    spawn_laser_glitch_child(world, white_id, laser_entity, allocator);
}

fn spawn_laser_glitch_child(
    world: &mut World,
    model_id: usize,
    laser_entity: Entity,
    allocator: &mut EntityAllocator,
) {
    world
        .spawn(allocator)
        .with(Renderable::new(model_id))
        .with(Transform::new())
        .with(GlitchVfx {
            max_offset: LASER_GLITCH_MAX_OFFSET,
            flash: None,
        })
        .as_child_of(laser_entity)
        .build();
}

fn spawn_laser(
    world: &mut World,
    asset_server: &AssetServer,
    position: Vector3<f32>,
    scale: Vector3<f32>,
    fired_at: Instant,
    player_z_speed: f32,
    allocator: &mut EntityAllocator,
) -> Entity {
    let laser_model_id = asset_server.get_model_id("laser");
    world
        .spawn(allocator)
        .with(Renderable::new(laser_model_id))
        .with(asset_server.get_collider_aabb("laser"))
        .with(Transform {
            position,
            scale,
            rotation: Quaternion::one(),
        })
        .with(Velocity {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .with(Laser {
            initial_z: position.z,
            fired_at,
            travel_speed: DEFAULT_TRAVEL_SPEED + player_z_speed,
        })
        .build()
}

