use cgmath::Vector3;

use crate::{
    engine::{
        ecs::{resources::toasts::{Toast, ToastQueue}, system::SystemContext, world::World},
        events::events::Events,
    },
    game::{
        events::enemy_killed_event::EnemyKilledEvent,
        resources::player_score::PlayerScore,
        systems::player_score_system::BASIC_KILL_SCORE,
    },
};

// Lift the pop above the enemy/explosion so it reads as a separate beat rather
// than overlapping the burst.
const POPUP_WORLD_Y_OFFSET: f32 = 1.0;

pub fn score_popup_system(world: &mut World, system_context: &mut SystemContext) {
    let kill_origins: Vec<Vector3<f32>> = world
        .get_resource::<Events<EnemyKilledEvent>>()
        .unwrap()
        .read()
        .filter(|event| event.show_popup)
        .map(|event| event.origin)
        .collect();
    if kill_origins.is_empty() {
        return;
    }

    // Mirror the per-kill award in player_score_system so the pop shows the
    // exact points granted, multiplier included.
    let per_kill = world
        .get_resource::<PlayerScore>()
        .map(|score| score.increment * score.multiplier * BASIC_KILL_SCORE)
        .unwrap_or(BASIC_KILL_SCORE);
    let text = format!("+{per_kill}");

    for origin in kill_origins {
        let position = origin + Vector3::new(0.0, POPUP_WORLD_Y_OFFSET, 0.0);
        let label = text.clone();
        system_context
            .commands()
            .update_resource::<ToastQueue, _>(move |queue| {
                queue.push(Toast::world(label, position));
            });
    }
}
