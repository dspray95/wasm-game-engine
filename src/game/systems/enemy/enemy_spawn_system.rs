use cgmath::{One, Quaternion, Vector3};
use rand::Rng;

use crate::{
    engine::ecs::{
        commands::commands::Commands,
        components::{
            collider::{Collider, ColliderShape},
            renderable::Renderable,
            transform::Transform,
            velocity::Velocity,
        },
        entity::{Entity, EntityAllocator},
        system::SystemContext,
        world::World,
    },
    game::{
        components::{
            dead::Dead,
            enemy::Enemy,
            hover_state::{HoverDirection, HoverState},
            player::Player,
        },
        resources::{enemy_resources::EnemySpawnManager, player_score::PlayerScore},
    },
};

const SPAWN_SINGLE_ROLL: f32 = 0.4;
const SPAWN_GATE_ROLL: f32 = 0.75;
const _SPAWN_STACK_ROLL: f32 = 1.0;

#[derive(Clone, Copy)]
enum Lane {
    Left,
    Center,
    Right,
}

impl Lane {
    fn x_offset(self, lane_offset: f32) -> f32 {
        match self {
            Lane::Left => -lane_offset,
            Lane::Center => 0.0,
            Lane::Right => lane_offset,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Lane::Left,
            1 => Lane::Center,
            _ => Lane::Right,
        }
    }
}

pub fn enemy_spawn_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity_id) = world.iter_component::<Player>().next().map(|(id, _)| id) else {
        return;
    };
    if world.get_component_by_id::<Dead>(player_entity_id).is_some() {
        return;
    }
    let player_position = world
        .get_component_by_id::<Transform>(player_entity_id)
        .map(|transform| transform.position);

    let Some(player_position) = player_position else {
        return;
    };

    let Some(manager) = world.get_resource::<EnemySpawnManager>() else {
        return;
    };

    let player_score = world
        .get_resource::<PlayerScore>()
        .map(|s| s.score)
        .unwrap_or(0);

    let current_interval = manager.spawn_interval.value(player_score);
    let next_time_since_last_spawn = manager.time_since_last_spawn + system_context.delta_time;
    let despawn_threshold = player_position.z - manager.despawn_behind_distance;
    let existing_enemies = manager.enemy_entities.clone();
    let canyon_center_x = manager.canyon_center_x;
    let lane_offset = manager.lane_offset;
    let column_z_offset = manager.column_z_offset;
    let spawn_horizon_z = manager.spawn_horizon_z;
    let spawn_elevation = manager.enemy_spawn_elevation;
    let spawn_scale = manager.enemy_spawn_scale;
    let should_spawn = next_time_since_last_spawn >= current_interval;

    let entities_to_despawn: Vec<Entity> = existing_enemies
        .into_iter()
        .filter_map(|entity| {
            let transform = world.get_component_by_id::<Transform>(entity.id)?;
            (transform.position.z < despawn_threshold).then_some(entity)
        })
        .collect();

    for entity in &entities_to_despawn {
        system_context.commands.despawn(*entity);
    }

    let mut spawned: Vec<Entity> = Vec::new();

    if should_spawn {
        let base_z = player_position.z + spawn_horizon_z;
        let starfighter_model_id = system_context
            .asset_server
            .as_deref()
            .unwrap()
            .get_model_id("starfighter_enemy");

        let mut rng = rand::rng();
        let pattern_roll: f32 = rng.random_range(0.0..1.0);

        let spawn_at = |lane: Lane,
                        z: f32,
                        commands: &mut Commands,
                        allocator: &mut EntityAllocator|
         -> Entity {
            spawn_enemy(
                commands,
                allocator,
                starfighter_model_id,
                Vector3 {
                    x: canyon_center_x + lane.x_offset(lane_offset),
                    y: spawn_elevation,
                    z,
                },
                spawn_scale,
            )
        };

        if pattern_roll < SPAWN_SINGLE_ROLL {
            // Single: one ship in a random lane
            let lane = Lane::from_index(rng.random_range(0..3));
            spawned.push(spawn_at(
                lane,
                base_z,
                system_context.commands,
                system_context.entity_allocator,
            ));
        } else if pattern_roll < SPAWN_GATE_ROLL {
            // Gate: two ships at same z, one lane left open
            let open_lane_index = rng.random_range(0..3);
            for i in 0..3 {
                if i == open_lane_index {
                    continue;
                }
                spawned.push(spawn_at(
                    Lane::from_index(i),
                    base_z,
                    system_context.commands,
                    system_context.entity_allocator,
                ));
            }
        } else {
            // Column: two ships in same lane, second offset further along z
            let lane = Lane::from_index(rng.random_range(0..3));
            spawned.push(spawn_at(
                lane,
                base_z,
                system_context.commands,
                system_context.entity_allocator,
            ));
            spawned.push(spawn_at(
                lane,
                base_z + column_z_offset,
                system_context.commands,
                system_context.entity_allocator,
            ));
        }
    }

    let spawned_count = spawned.len();
    let did_spawn = should_spawn;
    system_context
        .commands
        .update_resource::<EnemySpawnManager, _>(move |manager| {
            manager
                .enemy_entities
                .retain(|e| !entities_to_despawn.contains(e));
            for entity in &spawned {
                manager.enemy_entities.push(*entity);
            }
            manager.n_enemies_spawned += spawned_count;
            if did_spawn {
                manager.time_since_last_spawn = 0.0;
            } else {
                manager.time_since_last_spawn = next_time_since_last_spawn;
            }
        });
}

fn spawn_enemy(
    commands: &mut Commands,
    allocator: &mut EntityAllocator,
    model_id: usize,
    position: Vector3<f32>,
    scale: Vector3<f32>,
) -> Entity {
    commands
        .spawn(allocator)
        .with(Enemy)
        .with(Renderable::new(model_id))
        .with(Collider {
            shape: ColliderShape::AABB {
                offset: Vector3::new(0.0, 0.0, -0.3),
                half_extents: Vector3::new(1.0, 0.5, 1.5),
            },
        })
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
        .with(HoverState {
            direction: HoverDirection::Down,
            upper_limit: -0.9,
            lower_limit: -0.99,
        })
        .build()
}
