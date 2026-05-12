use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::{events::score_event::ScoreEvent, resources::player_score::PlayerScore},
};

pub fn player_score_system(world: &mut World, system_context: &mut SystemContext) {
    let dt = system_context.delta_time;
    let event_count = world.events::<ScoreEvent>().count();

    system_context
        .commands()
        .update_resource::<PlayerScore, _>(move |score| {
            score.time_since_last_increment += dt;
            if score.time_since_last_increment >= score.increment_interval_seconds {
                score.time_since_last_increment -= score.increment_interval_seconds;
                score.score += score.increment * score.multiplier;
            }
            for _ in 0..event_count {
                score.score += score.increment * score.multiplier;
            }
        });
}
