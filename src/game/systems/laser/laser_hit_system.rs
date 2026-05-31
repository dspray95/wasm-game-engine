use std::collections::HashSet;

use cgmath::{One, Quaternion, Vector3};
use rand::Rng;

use crate::{
    engine::{
        ecs::{
            components::{
                collider::{Collider, ColliderShape},
                renderable::Renderable,
                transform::Transform,
                world_transform::WorldTransform,
            },
            entity::Entity,
            events::collision_event::CollisionEvent,
            system::SystemContext,
            systems::collision_system::filter_collision_pairs,
            world::World,
        },
        events::events::Events,
    },
    game::{
        components::{
            enemy::Enemy,
            hover_state::{HoverDirection, HoverState},
            laser::Laser,
            pickup::Pickup,
        },
        events::{
            enemy_killed_event::EnemyKilledEvent,
            score_event::{ScoreEvent, ScoreType},
        },
        resources::{enemy_resources::EnemySpawnManager, laser_resources::LaserManager},
    },
};

const PICKUP_DROP_CHANCE: f32 = 0.20;
const PICKUP_SCALE: f32 = 0.25;

pub fn laser_hit_system(world: &mut World, system_context: &mut SystemContext) {
    let collision_events: Vec<(Entity, Entity)> = world
        .get_resource::<Events<CollisionEvent>>()
        .unwrap()
        .read()
        .map(|event| (event.a, event.b))
        .collect();

    let hits = filter_collision_pairs::<Laser, Enemy>(world, &collision_events);

    let lasers_to_despawn: HashSet<Entity> = hits.iter().map(|(laser, _)| *laser).collect();
    let enemies_to_despawn: HashSet<Entity> = hits.iter().map(|(_, enemy)| *enemy).collect();

    let enemy_killed_events: Vec<EnemyKilledEvent> = enemies_to_despawn
        .iter()
        .filter_map(|enemy| {
            world
                .get_component_by_id::<WorldTransform>(enemy.id)
                .map(|t| EnemyKilledEvent {
                    origin: t.position,
                    show_popup: true,
                })
        })
        .collect();

    {
        let lasers = lasers_to_despawn.clone();
        system_context
            .commands()
            .update_resource::<LaserManager, _>(move |m| {
                m.alive_lasers.retain(|e| !lasers.contains(e));
            });
    }
    {
        let enemies = enemies_to_despawn.clone();
        system_context
            .commands()
            .update_resource::<EnemySpawnManager, _>(move |m| {
                m.enemy_entities.retain(|e| !enemies.contains(e));
            });
    }

    for entity in lasers_to_despawn.into_iter().chain(enemies_to_despawn) {
        system_context.commands().despawn(entity);
    }

    for _ in 0..enemy_killed_events.len() {
        system_context.commands().send_event(ScoreEvent {
            score_type: ScoreType::EnemyKilled,
        });
    }

    let pickup_model_id = system_context
        .asset_server
        .as_deref()
        .map(|server| server.get_model_id("pickup"));

    let mut rng = rand::rng();
    for event in &enemy_killed_events {
        if let Some(model_id) = pickup_model_id {
            if rng.random::<f32>() < PICKUP_DROP_CHANCE {
                spawn_pickup(system_context, model_id, event.origin);
            }
        }
    }

    for event in enemy_killed_events {
        system_context.commands().send_event(event);
    }
}

fn spawn_pickup(
    system_context: &mut SystemContext,
    model_id: usize,
    position: Vector3<f32>,
) {
    system_context
        .commands
        .spawn(system_context.entity_allocator)
        .with(Pickup)
        .with(Renderable::new(model_id))
        .with(Transform {
            position,
            scale: Vector3 {
                x: PICKUP_SCALE,
                y: PICKUP_SCALE,
                z: PICKUP_SCALE,
            },
            rotation: Quaternion::one(),
        })
        .with(Collider {
            shape: ColliderShape::AABB {
                offset: Vector3::new(0.0, 0.0, 0.0),
                half_extents: Vector3::new(0.5, 0.5, 0.5),
            },
        })
        .with(HoverState {
            direction: HoverDirection::Up,
            upper_limit: position.y + 0.15,
            lower_limit: position.y - 0.15,
        })
        .build();
}
