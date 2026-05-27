use std::sync::Arc;

use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::resources::{
        async_tasks::AsyncHighScoreTask,
        high_scores::{HighScoreEntry, HighScores, MAX_ENTRIES},
    },
};

pub fn high_score_sync_system(world: &mut World, _system_context: &mut SystemContext) {
    let (fetch_arc, submit_arc) = match world.get_resource::<AsyncHighScoreTask>() {
        Some(task) => (Arc::clone(&task.fetch_result), Arc::clone(&task.submit_result)),
        None => return,
    };

    if let Ok(mut slot) = fetch_arc.lock() {
        if let Some(result) = slot.take() {
            match result {
                Ok(entries) => {
                    if let Some(high_scores) = world.get_resource_mut::<HighScores>() {
                        high_scores.entries = entries
                            .into_iter()
                            .map(|remote| HighScoreEntry {
                                initials: remote.initials,
                                score: remote.score,
                            })
                            .take(MAX_ENTRIES)
                            .collect();
                        high_scores.entries.sort_by(|a, b| b.score.cmp(&a.score));
                    }
                }
                Err(error) => log::warn!("high_score_sync_system: fetch failed: {error}"),
            }
        }
    }

    let should_refetch = if let Ok(mut slot) = submit_arc.lock() {
        if let Some(result) = slot.take() {
            match result {
                Ok(()) => true,
                Err(error) => {
                    log::warn!("high_score_sync_system: submit failed: {error}");
                    false
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    if should_refetch {
        request_fetch(world);
    }
}

pub fn request_fetch(world: &World) {
    let fetch_arc = match world.get_resource::<AsyncHighScoreTask>() {
        Some(task) => Arc::clone(&task.fetch_result),
        None => return,
    };

    #[cfg(target_arch = "wasm32")]
    {
        use crate::game::resources::firestore::fetch_high_scores;
        wasm_bindgen_futures::spawn_local(async move {
            let result = fetch_high_scores(MAX_ENTRIES as u32).await;
            if let Ok(mut slot) = fetch_arc.lock() {
                *slot = Some(result);
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: no-op; local file already populated HighScores on load.
        let _ = fetch_arc;
    }
}

pub fn request_submit(world: &World, initials: String, score: i32) {
    let submit_arc = match world.get_resource::<AsyncHighScoreTask>() {
        Some(task) => Arc::clone(&task.submit_result),
        None => return,
    };

    #[cfg(target_arch = "wasm32")]
    {
        use crate::game::resources::firestore::submit_high_score;
        wasm_bindgen_futures::spawn_local(async move {
            let result = submit_high_score(&initials, score).await;
            if let Ok(mut slot) = submit_arc.lock() {
                *slot = Some(result);
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (submit_arc, initials, score);
    }
}
