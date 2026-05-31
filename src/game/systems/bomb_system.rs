use crate::{
    engine::ecs::{
        components::world_transform::WorldTransform,
        entity::Entity,
        resources::toasts::push_hud_toast,
        system::SystemContext,
        world::World,
    },
    game::{
        components::{bomb_charging::BombCharging, player::Player},
        events::{
            enemy_killed_event::EnemyKilledEvent,
            score_event::{ScoreEvent, ScoreType},
        },
        resources::{
            enemy_resources::EnemySpawnManager, player_score::PlayerScore,
            screen_effects::{
                ScreenEffects, SHAKE_BIG_DURATION_SECONDS, SHAKE_BIG_MAGNITUDE,
                SHAKE_SUPER_MAGNITUDE,
            },
        },
        systems::pickup_collect_system::clear_all_powerups,
    },
};

const BOMB_BONUS_SCORE: i32 = 500;
/// Per-enemy score the bomb counts toward the on-screen total. Must match
/// `BASIC_KILL_SCORE` in player_score_system so the toast matches what the
/// player_score_system actually awards via ScoreEvents.
const BOMB_PER_KILL_SCORE: i32 = 25;
const WHITE_FLASH_SECONDS: f32 = 0.4;

pub fn bomb_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    let Some((new_time, total_seconds)) = world
        .get_component::<BombCharging>(player_entity)
        .map(|b| (b.time_remaining - system_context.delta_time, b.total_seconds))
    else {
        return;
    };

    if new_time > 0.0 {
        // Mid-charge: ramp shake from BIG → SUPER as the timer winds down.
        // We re-trigger the shake each frame so the magnitude lerp takes
        // effect and the shake never naturally times out mid-charge.
        let progress = (1.0 - (new_time / total_seconds)).clamp(0.0, 1.0);
        let magnitude =
            SHAKE_BIG_MAGNITUDE + (SHAKE_SUPER_MAGNITUDE - SHAKE_BIG_MAGNITUDE) * progress;
        system_context
            .commands
            .update_resource::<ScreenEffects, _>(move |s| {
                s.trigger_shake(SHAKE_BIG_DURATION_SECONDS, magnitude);
            });
        system_context
            .commands
            .update_component::<BombCharging, _>(player_entity, move |b| {
                b.time_remaining = new_time;
            });
        return;
    }

    // Detonation: count alive enemies for the score total, despawn each
    // through the existing kill-event plumbing (so explosions + per-kill
    // ScoreEvents fire normally), then layer the bomb-specific bonus, white
    // flash, score toast, and chain reset on top.
    let enemies: Vec<Entity> = world
        .get_resource::<EnemySpawnManager>()
        .map(|m| m.enemy_entities.clone())
        .unwrap_or_default();
    let kill_count = enemies.len() as i32;
    let multiplier = world
        .get_resource::<PlayerScore>()
        .map(|s| s.multiplier)
        .unwrap_or(1);
    let total_score = (kill_count * BOMB_PER_KILL_SCORE + BOMB_BONUS_SCORE) * multiplier;

    for enemy in &enemies {
        if let Some(transform) = world.get_component_by_id::<WorldTransform>(enemy.id) {
            system_context.commands.send_event(EnemyKilledEvent {
                origin: transform.position,
                // Bomb shows one aggregated HUD total, not a pop per enemy.
                show_popup: false,
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

    // Bonus score: add directly so the multiplier still applies via the
    // PlayerScore.multiplier field (matches the pattern used by
    // player_score_system for kill scoring).
    system_context
        .commands
        .update_resource::<PlayerScore, _>(|score| {
            score.score += score.increment * score.multiplier * BOMB_BONUS_SCORE;
        });

    system_context
        .commands
        .update_resource::<ScreenEffects, _>(|s| {
            s.trigger_white_flash(WHITE_FLASH_SECONDS);
        });

    push_hud_toast(system_context.commands, format!("+{}!!!", total_score));

    clear_all_powerups(world, system_context, player_entity);
    system_context
        .commands
        .remove_component::<BombCharging>(player_entity);
}
