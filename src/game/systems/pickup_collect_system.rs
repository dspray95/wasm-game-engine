use std::collections::HashSet;

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use crate::{
    engine::{
        ecs::{
            components::{
                renderable::Renderable, transform::Transform, world_transform::WorldTransform,
            },
            entity::{Entity, EntityAllocator},
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
            double_fire_rate::DoubleFireRate, glitch_vfx::GlitchVfx, hyperdrive::Hyperdrive,
            pickup::Pickup, player::Player, shield::Shield,
        },
        events::{
            enemy_killed_event::EnemyKilledEvent,
            score_event::{ScoreEvent, ScoreType},
        },
        resources::{
            enemy_resources::EnemySpawnManager,
            player_score::PlayerScore,
            powerup_progression::{next_powerup_kind, PowerUpKind},
            screen_effects::ScreenEffects,
        },
    },
};

// All three durations match so a fully-stacked grab refreshes everything to
// the same expiry — they tick down in lockstep and the stack collapses cleanly
// rather than peeling layers off at staggered moments.
pub const POWERUP_DURATION_SECONDS: f32 = 7.5;
pub const SHIELD_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;
pub const HYPERDRIVE_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;
pub const DOUBLE_FIRE_RATE_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;

// Hyperdrive glitch — continuous shoogle (no flash schedule) for a chromatic
// speed-blur look while the powerup is active. Bigger offset than the
// laser's so it reads at player scale (0.3).
const PLAYER_GLITCH_MAX_OFFSET: f32 = 0.4;
const SHIELD_SCALE: f32 = 1.25;
const SHIELD_FLASH_PHASE_SECONDS: f32 = 0.1;

pub fn pickup_collect_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    let collision_events: Vec<(Entity, Entity)> = world
        .get_resource::<Events<CollisionEvent>>()
        .unwrap()
        .read()
        .map(|event| (event.a, event.b))
        .collect();

    let hits = filter_collision_pairs::<Player, Pickup>(world, &collision_events);
    if hits.is_empty() {
        return;
    }

    let pickups_to_despawn: HashSet<Entity> = hits.iter().map(|(_, pickup)| *pickup).collect();
    for pickup in &pickups_to_despawn {
        system_context.commands.despawn(*pickup);
    }

    let kind = next_powerup_kind(world, player_entity);

    match kind {
        PowerUpKind::Shield => {
            let mut rng = rand::rng();
            grant_shield(system_context, player_entity, &mut rng);
            refresh_active_timers(world, system_context, player_entity);
            push_hud_toast(system_context.commands, "SHIELD ENABLED");
            push_hud_toast(system_context.commands, "LASER NEXT");
        }
        PowerUpKind::Laser => {
            grant_double_fire_rate(system_context, player_entity);
            refresh_active_timers(world, system_context, player_entity);
            push_hud_toast(system_context.commands, "LASER ENABLED");
            push_hud_toast(system_context.commands, "HYPERDRIVE NEXT");
        }
        PowerUpKind::Hyperdrive => {
            grant_hyperdrive(system_context, player_entity);
            refresh_active_timers(world, system_context, player_entity);
            push_hud_toast(system_context.commands, "HYPERDRIVE ENABLED");
            push_hud_toast(system_context.commands, "BOMB READY");
        }
        PowerUpKind::Bomb => {
            // Bomb caps the chain: nuke everything on screen, clear every
            // active powerup as "payment", chain resets to empty stack so the
            // next pickup gives Shield again.
            trigger_bomb(world, system_context);
            clear_all_powerups(world, system_context, player_entity);
            push_hud_toast(system_context.commands, "BOMB!");
        }
    }
}

/// Bump every active powerup's `time_remaining` back to its full duration so
/// the stack stays synchronised. Called after every grant so existing
/// powerups don't expire mid-stack-build.
fn refresh_active_timers(
    world: &World,
    system_context: &mut SystemContext,
    player_entity: Entity,
) {
    if world.get_component::<Shield>(player_entity).is_some() {
        system_context
            .commands
            .update_component::<Shield, _>(player_entity, |s| {
                s.time_remaining = SHIELD_DURATION_SECONDS;
            });
    }
    if world
        .get_component::<DoubleFireRate>(player_entity)
        .is_some()
    {
        system_context
            .commands
            .update_component::<DoubleFireRate, _>(player_entity, |d| {
                d.time_remaining = DOUBLE_FIRE_RATE_DURATION_SECONDS;
            });
    }
    if world.get_component::<Hyperdrive>(player_entity).is_some() {
        system_context
            .commands
            .update_component::<Hyperdrive, _>(player_entity, |h| {
                h.time_remaining = HYPERDRIVE_DURATION_SECONDS;
            });
    }
}

/// Despawn visuals + remove every powerup component from the player +
/// undo side-effects (score multiplier). Used by bomb (chain cashed out) and
/// by player_damage_system (chain reset on hyperdrive-active hit).
pub fn clear_all_powerups(
    world: &World,
    system_context: &mut SystemContext,
    player_entity: Entity,
) {
    if let Some(shield_visual) = world
        .get_component::<Shield>(player_entity)
        .map(|s| s.visual_entity)
    {
        system_context
            .commands
            .remove_component::<Shield>(player_entity);
        system_context.commands.despawn(shield_visual);
    }

    if let Some(glitch_visuals) = world
        .get_component::<Hyperdrive>(player_entity)
        .map(|h| h.glitch_visuals.clone())
    {
        system_context
            .commands
            .remove_component::<Hyperdrive>(player_entity);
        system_context
            .commands
            .update_resource::<PlayerScore, _>(|score| {
                score.multiplier = 1;
            });
        for visual in glitch_visuals {
            system_context.commands.despawn(visual);
        }
    }

    if world
        .get_component::<DoubleFireRate>(player_entity)
        .is_some()
    {
        system_context
            .commands
            .remove_component::<DoubleFireRate>(player_entity);
    }
}

/// Instant clear of every alive enemy — sends an EnemyKilledEvent + ScoreEvent
/// per enemy (reusing the existing explosion + score plumbing), despawns them,
/// and triggers a beefy screen shake.
fn trigger_bomb(world: &World, system_context: &mut SystemContext) {
    let enemy_entities: Vec<Entity> = world
        .get_resource::<EnemySpawnManager>()
        .map(|m| m.enemy_entities.clone())
        .unwrap_or_default();

    for enemy in &enemy_entities {
        if let Some(transform) = world.get_component_by_id::<WorldTransform>(enemy.id) {
            system_context.commands.send_event(EnemyKilledEvent {
                origin: transform.position,
            });
            system_context.commands.send_event(ScoreEvent {
                score_type: ScoreType::EnemyKilled,
            });
        }
        system_context.commands.despawn(*enemy);
    }

    system_context
        .commands
        .update_resource::<EnemySpawnManager, _>(|m| {
            m.enemy_entities.clear();
        });

    system_context
        .commands
        .update_resource::<ScreenEffects, _>(|s| {
            s.trigger_damage_effect();
        });
}

fn grant_shield(system_context: &mut SystemContext, player_entity: Entity, rng: &mut impl Rng) {
    let Some(shield_model_id) = system_context
        .asset_server
        .as_deref()
        .map(|server| server.get_model_id("shield"))
    else {
        return;
    };

    // Shield visual spawns as a child of the player with identity local
    // Transform — the hierarchy_system composes it onto the player's world
    // position every frame, no per-frame sync needed.
    let visual_entity = spawn_shield_visual(
        system_context.commands,
        system_context.entity_allocator,
        shield_model_id,
        player_entity,
    );

    let rotation_axis = Vector3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    )
    .normalize();

    system_context.commands.add_component(
        player_entity,
        Shield {
            time_remaining: SHIELD_DURATION_SECONDS,
            visual_entity,
            rotation_axis,
            flash_phase_timer: SHIELD_FLASH_PHASE_SECONDS,
        },
    );
}

fn spawn_shield_visual(
    commands: &mut crate::engine::ecs::commands::commands::Commands,
    allocator: &mut EntityAllocator,
    shield_model_id: usize,
    player_entity: Entity,
) -> Entity {
    commands
        .spawn(allocator)
        .with(Renderable::new(shield_model_id))
        .with(Transform::new().with_scale(SHIELD_SCALE, SHIELD_SCALE, SHIELD_SCALE))
        .as_child_of(player_entity)
        .build()
}

fn grant_hyperdrive(system_context: &mut SystemContext, player_entity: Entity) {
    let glitch_visuals = spawn_ship_glitch_pair(system_context, player_entity);
    system_context.commands.add_component(
        player_entity,
        Hyperdrive {
            time_remaining: HYPERDRIVE_DURATION_SECONDS,
            glitch_visuals,
        },
    );
    system_context
        .commands
        .update_resource::<PlayerScore, _>(|score| {
            score.multiplier = 2;
        });
}

fn spawn_ship_glitch_pair(
    system_context: &mut SystemContext,
    player_entity: Entity,
) -> Vec<Entity> {
    let Some(asset_server) = system_context.asset_server.as_deref() else {
        return Vec::new();
    };
    let cyan_model_id = asset_server.get_model_id("starfighter_glitch_cyan");
    let white_model_id = asset_server.get_model_id("starfighter_glitch_white");
    let cyan = spawn_glitch_child(
        system_context.commands,
        system_context.entity_allocator,
        cyan_model_id,
        player_entity,
        PLAYER_GLITCH_MAX_OFFSET,
    );
    let white = spawn_glitch_child(
        system_context.commands,
        system_context.entity_allocator,
        white_model_id,
        player_entity,
        PLAYER_GLITCH_MAX_OFFSET,
    );
    vec![cyan, white]
}

fn spawn_glitch_child(
    commands: &mut crate::engine::ecs::commands::commands::Commands,
    allocator: &mut EntityAllocator,
    model_id: usize,
    parent_entity: Entity,
    max_offset: f32,
) -> Entity {
    commands
        .spawn(allocator)
        .with(Renderable::new(model_id))
        .with(Transform::new())
        .with(GlitchVfx {
            max_offset,
            flash: None,
        })
        .as_child_of(parent_entity)
        .build()
}

fn grant_double_fire_rate(system_context: &mut SystemContext, player_entity: Entity) {
    system_context.commands.add_component(
        player_entity,
        DoubleFireRate {
            time_remaining: DOUBLE_FIRE_RATE_DURATION_SECONDS,
        },
    );
}
