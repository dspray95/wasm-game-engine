use std::collections::HashSet;

use cgmath::Vector3;

use crate::{
    engine::{
        ecs::{
            components::{renderable::Renderable, world_transform::WorldTransform},
            entity::Entity,
            events::collision_event::CollisionEvent,
            resources::toasts::push_hud_toast,
            system::SystemContext,
            systems::collision_system::filter_collision_pairs,
            world::World,
        },
        events::events::Events,
    },
    game::{
        components::{
            dead::Dead, enemy::Enemy, hyperdrive::Hyperdrive, invulnerable::Invulnerable,
            player::Player, shield::Shield,
        },
        events::{
            enemy_killed_event::EnemyKilledEvent,
            player_died_event::PlayerDiedEvent,
            score_event::{ScoreEvent, ScoreType},
        },
        helpers::player_speed::effective_player_z_speed,
        resources::{enemy_resources::EnemySpawnManager, screen_effects::ScreenEffects},
        systems::pickup_collect_system::clear_all_powerups,
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
    let shield_visual = world
        .get_component::<Shield>(player_entity)
        .map(|s| s.visual_entity);
    let shielded = shield_visual.is_some();
    let hyperdrive_active = world.get_component::<Hyperdrive>(player_entity).is_some();

    let player_position = world
        .get_component::<WorldTransform>(player_entity)
        .map(|t| t.position);

    let enemy_kill_events: Vec<EnemyKilledEvent> = enemies_to_despawn
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
    let enemy_kill_count = enemy_kill_events.len();

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
        .update_resource::<ScreenEffects, _>(|effects| {
            effects.trigger_damage_effect();
        });

    if shielded {
        // Hit while shielded: consume the shield only. No HP loss, no
        // invulnerable grace period — the player needs to immediately
        // worry about the *next* hit. Other powerups (Laser, Hyperdrive)
        // stay active. Each enemy the shield broke through still counts
        // as a clean kill for score.
        for _ in 0..enemy_kill_count {
            system_context.commands().send_event(ScoreEvent {
                score_type: ScoreType::EnemyKilled,
            });
        }
        system_context
            .commands()
            .remove_component::<Shield>(player_entity);
        if let Some(visual) = shield_visual {
            system_context.commands().despawn(visual);
        }
        push_hud_toast(system_context.commands(), "SHIELD BROKEN");
        return;
    }

    // Unshielded hit — HP comes off. If Hyperdrive was active, this is the
    // "again" damage that resets the whole chain (rule 4): clear every
    // remaining powerup as the catastrophic cost.
    let new_health = player_health - 1;
    system_context
        .commands()
        .update_component::<Player, _>(player_entity, move |p| {
            p.health = new_health;
        });

    if hyperdrive_active {
        clear_all_powerups(world, system_context, player_entity);
        push_hud_toast(system_context.commands(), "CHAIN RESET");
    }

    if new_health <= 0 {
        let speed_at_death = effective_player_z_speed(world);
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
