use crate::{engine::ecs::entity::Entity, game::difficulty::DifficultyCurve};
use web_time::Instant;

const MAX_TRAVEL_DISTANCE: f32 = 50.0;

pub struct LaserManager {
    pub alive_lasers: Vec<Entity>,
    pub last_fired_time: Instant,
    pub fire_cooldown: DifficultyCurve,
    pub max_travel_distance: f32,
}

impl LaserManager {
    pub fn new() -> Self {
        Self {
            last_fired_time: Instant::now(),
            alive_lasers: Vec::new(),
            fire_cooldown: DifficultyCurve {
                base: 0.75,
                cap: 0.375,
                warmup_score: 0.0,
                full_scale_score: 1500.0,
                exponent: 0.7,
            },
            max_travel_distance: MAX_TRAVEL_DISTANCE,
        }
    }

    pub fn is_allowed_to_fire(&self, current_time: Instant, cooldown_seconds: f32) -> bool {
        let time_since_last_fire = current_time
            .duration_since(self.last_fired_time)
            .as_secs_f32();
        time_since_last_fire > cooldown_seconds
    }
}
