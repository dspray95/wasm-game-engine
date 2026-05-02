use crate::engine::ecs::entity::Entity;
use web_time::Instant;

const MAX_TRAVEL_DISTANCE: f32 = 50.0;
const FIRE_COOLDOWN_SECONDS: f32 = 0.75;

pub struct LaserManager {
    pub alive_lasers: Vec<Entity>,
    pub last_fired_time: Instant,
    pub fire_cooldown_seconds: f32,
    pub max_travel_distance: f32,
}

impl LaserManager {
    pub fn new() -> Self {
        Self {
            last_fired_time: Instant::now(),
            alive_lasers: Vec::new(),
            fire_cooldown_seconds: FIRE_COOLDOWN_SECONDS,
            max_travel_distance: MAX_TRAVEL_DISTANCE,
        }
    }

    pub fn is_allowed_to_fire(&self, current_time: Instant) -> bool {
        let time_since_last_fire = current_time.duration_since(self.last_fired_time).as_secs_f32();
        log::info!(
            "Checking time to fire - time since last: {:?}, cooldown_seconds: {:?}",
            time_since_last_fire,
            self.fire_cooldown_seconds
        );
        time_since_last_fire > self.fire_cooldown_seconds
    }
}
