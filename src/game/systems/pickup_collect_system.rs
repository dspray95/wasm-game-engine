use std::collections::HashSet;

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use crate::{
    engine::{
        ecs::{
            components::{renderable::Renderable, transform::Transform},
            entity::{Entity, EntityAllocator},
            events::collision_event::CollisionEvent,
            resources::toasts::{clear_hud_toasts, push_hud_toast, push_hud_toast_delayed},
            system::SystemContext,
            systems::collision_system::filter_collision_pairs,
            world::World,
        },
        events::events::Events,
    },
    game::{
        components::{
            bomb_charging::BombCharging, double_fire_rate::DoubleFireRate, glitch_vfx::GlitchVfx,
            hyperdrive::Hyperdrive, invulnerable::Invulnerable, pickup::Pickup, player::Player,
            shield::Shield,
        },
        resources::{
            player_score::PlayerScore,
            powerup_progression::{next_powerup_kind, PowerUpKind},
        },
    },
};

// All three durations match so a fully-stacked grab refreshes everything to
// the same expiry — they tick down in lockstep and the stack collapses cleanly
// rather than peeling layers off at staggered moments.
pub const POWERUP_DURATION_SECONDS: f32 = 7.5;
/// Delay between the "X ENABLED" toast and its "Y NEXT" companion so they
/// read as two beats rather than a single line.
const FOLLOWUP_TOAST_DELAY: f32 = 0.5;
pub const SHIELD_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;
pub const HYPERDRIVE_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;
pub const DOUBLE_FIRE_RATE_DURATION_SECONDS: f32 = POWERUP_DURATION_SECONDS;

// Hyperdrive glitch — continuous shoogle (no flash schedule) for a chromatic
// speed-blur look while the powerup is active. Bigger offset than the
// laser's so it reads at player scale (0.3).
const PLAYER_GLITCH_MAX_OFFSET: f32 = 0.4;
const SHIELD_SCALE: f32 = 1.25;
const SHIELD_FLASH_PHASE_SECONDS: f32 = 0.1;
/// Wind-up between grabbing a bomb pickup and the detonation. `bomb_system`
/// owns the ramp + fire; we just attach the timer here.
pub const BOMB_CHARGE_SECONDS: f32 = 2.0;
/// Flash period applied via `Invulnerable` during the wind-up — reuses the
/// existing flashing behaviour so the player gets the "I am invincible right
/// now" visual.
const BOMB_FLASH_PHASE_SECONDS: f32 = 0.08;

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

    // Mid-bomb-charge: pickup interactions are paused entirely so the player
    // can't grab another bomb on top of the wind-up.
    if world.get_component::<BombCharging>(player_entity).is_some() {
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
            push_hud_toast_delayed(system_context.commands, "LASER NEXT", FOLLOWUP_TOAST_DELAY);
        }
        PowerUpKind::Laser => {
            grant_double_fire_rate(system_context, player_entity);
            refresh_active_timers(world, system_context, player_entity);
            // Drop the stale "LASER NEXT" that the previous Shield grab queued.
            clear_hud_toasts(system_context.commands, "LASER NEXT");
            push_hud_toast(system_context.commands, "LASER ENABLED");
            push_hud_toast_delayed(
                system_context.commands,
                "HYPERDRIVE NEXT",
                FOLLOWUP_TOAST_DELAY,
            );
        }
        PowerUpKind::Hyperdrive => {
            grant_hyperdrive(system_context, player_entity);
            refresh_active_timers(world, system_context, player_entity);
            clear_hud_toasts(system_context.commands, "HYPERDRIVE NEXT");
            push_hud_toast(system_context.commands, "HYPERDRIVE ENABLED");
            push_hud_toast_delayed(system_context.commands, "BOMB READY", FOLLOWUP_TOAST_DELAY);
        }
        PowerUpKind::Bomb => {
            // Bomb is a 2-second wind-up, not an instant nuke. Attach the
            // charge timer and an Invulnerable so the player flashes and
            // can't be hit during the build-up. `bomb_system` reads
            // BombCharging, ramps the shake, and detonates at expiry.
            system_context.commands.add_component(
                player_entity,
                BombCharging {
                    time_remaining: BOMB_CHARGE_SECONDS,
                    total_seconds: BOMB_CHARGE_SECONDS,
                },
            );
            system_context.commands.add_component(
                player_entity,
                Invulnerable {
                    time_remaining: BOMB_CHARGE_SECONDS,
                    flash_phase_timer: BOMB_FLASH_PHASE_SECONDS,
                },
            );
            clear_hud_toasts(system_context.commands, "BOMB READY");
            push_hud_toast(system_context.commands, "BOMB INCOMING");
        }
    }
}

/// Bump every active powerup's `time_remaining` back to its full duration so
/// the stack stays synchronised. Called after every grant so existing
/// powerups don't expire mid-stack-build.
fn refresh_active_timers(world: &World, system_context: &mut SystemContext, player_entity: Entity) {
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
