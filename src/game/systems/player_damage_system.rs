use std::collections::HashSet;

use cgmath::Vector3;

use crate::{
    engine::{
        ecs::{
            components::{renderable::Renderable, transform::Transform},
            entity::Entity,
            events::collision_event::CollisionEvent, system::SystemContext,
            systems::collision_system::filter_collision_pairs, world::World,
        },
        events::events::Events,
    },
    game::{
        components::{
            dead::Dead, enemy::Enemy, invulnerable::Invulnerable, player::Player,
        },
        events::{
            enemy_killed_event::EnemyKilledEvent, player_died_event::PlayerDiedEvent,
        },
        resources::{
            enemy_resources::EnemySpawnManager, player_score::PlayerScore,
            player_speed_scaling::PlayerSpeedScaling, screen_effects::ScreenEffects,
        },
    },
};

const INVULNERABILITY_SECONDS: f32 = 2.0;
const FLASH_PHASE_SECONDS: f32 = 0.1;
const DEATH_SPEED_RAMP_SECONDS: f32 = 1.0;

pub fn player_damage_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    if world.get_component::<Dead>(player_entity).is_some() {
        return;
    }
    if world.get_component::<Invulnerable>(player_entity).is_some() {
        return;
    }

    let Some(player_health) = world.get_component::<Player>(player_entity).map(|p| p.health) else {
        return;
    };

    let collision_events: Vec<(Entity, Entity)> = world
        .get_resource::<Events<CollisionEvent>>()
        .unwrap()
        .read()
        .map(|event| (event.a, event.b))
        .collect();

    let hits = filter_collision_pairs::<Player, Enemy>(world, &collision_events);
    if hits.is_empty() {
        return;
    }

    let enemies_to_despawn: HashSet<Entity> = hits.iter().map(|(_, enemy)| *enemy).collect();
    let new_health = player_health - 1;

    let player_position = world
        .get_component::<Transform>(player_entity)
        .map(|t| t.position);

    let enemy_kill_events: Vec<EnemyKilledEvent> = enemies_to_despawn
        .iter()
        .filter_map(|enemy| {
            world
                .get_component_by_id::<Transform>(enemy.id)
                .map(|t| EnemyKilledEvent { origin: t.position })
        })
        .collect();

    {
        let enemies = enemies_to_despawn.clone();
        system_context
            .commands()
            .update_resource::<EnemySpawnManager, _>(move |m| {
                m.enemy_entities.retain(|e| !enemies.contains(e));
            });
    }
    for enemy in enemies_to_despawn {
        system_context.commands().despawn(enemy);
    }
    for event in enemy_kill_events {
        system_context.commands().send_event(event);
    }

    system_context
        .commands()
        .update_component::<Player, _>(player_entity, move |p| {
            p.health = new_health;
        });

    system_context
        .commands()
        .update_resource::<ScreenEffects, _>(|effects| {
            effects.trigger_damage_effect();
        });

    if new_health <= 0 {
        let speed_at_death = current_player_speed(world);
        if let Some(origin) = player_position {
            system_context.commands().send_event(PlayerDiedEvent {
                origin,
                velocity: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: speed_at_death,
                },
            });
        }
        system_context.commands().add_component(
            player_entity,
            Dead {
                speed_at_death,
                ramp_time_remaining: DEATH_SPEED_RAMP_SECONDS,
                ramp_duration: DEATH_SPEED_RAMP_SECONDS,
            },
        );
        system_context
            .commands()
            .update_component::<Renderable, _>(player_entity, |r| {
                r.visible = false;
            });
    } else {
        system_context.commands().add_component(
            player_entity,
            Invulnerable {
                time_remaining: INVULNERABILITY_SECONDS,
                flash_phase_timer: FLASH_PHASE_SECONDS,
            },
        );
    }
}

fn current_player_speed(world: &World) -> f32 {
    let score = world
        .get_resource::<PlayerScore>()
        .map(|s| s.score)
        .unwrap_or(0);
    match world.get_resource::<PlayerSpeedScaling>() {
        Some(scaling) => scaling.z_speed.value(score),
        None => 0.0,
    }
}
