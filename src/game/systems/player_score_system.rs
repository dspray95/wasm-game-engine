use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::{
        events::score_event::{ScoreEvent, ScoreType},
        resources::player_score::PlayerScore,
    },
};

pub fn player_score_system(world: &mut World, system_context: &mut SystemContext) {
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
            player_score.score +=
                player_score.increment * player_score.multiplier * 10 * basic_kill_count;
        });
}
