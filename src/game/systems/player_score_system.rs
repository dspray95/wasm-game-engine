use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::{
        components::{dead::Dead, player::Player},
        events::score_event::{ScoreEvent, ScoreType},
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            player_score::PlayerScore,
            tutorial_state::TutorialState,
        },
    },
};

pub const BASIC_KILL_SCORE: i32 = 25;

pub fn player_score_system(world: &mut World, system_context: &mut SystemContext) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|s| s.phase)
        .unwrap_or(GameOverPhase::Playing);
    if !matches!(phase, GameOverPhase::Playing) {
        return;
    }

    // No score until the tutorial is dismissed, matching the enemy-spawn hold.
    let tutorial_done = world
        .get_resource::<TutorialState>()
        .map(|s| s.completed)
        .unwrap_or(true);
    if !tutorial_done {
        return;
    }

    let player_dead = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| world.get_component_by_id::<Dead>(id).is_some())
        .unwrap_or(false);
    if player_dead {
        return;
    }

    let dt: f32 = system_context.delta_time;
    let basic_kill_count = world
        .events::<ScoreEvent>()
        .filter(|e| e.score_type == ScoreType::EnemyKilled)
        .count() as i32;

    system_context
        .commands()
        .update_resource::<PlayerScore, _>(move |player_score| {
            player_score.time_since_last_increment += dt;
            if player_score.time_since_last_increment >= player_score.increment_interval_seconds {
                player_score.time_since_last_increment -= player_score.increment_interval_seconds;
                player_score.score += player_score.increment * player_score.multiplier;
            }
            player_score.score += player_score.increment
                * player_score.multiplier
                * BASIC_KILL_SCORE
                * basic_kill_count;
        });
}
