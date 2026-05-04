use serde::{Deserialize, Serialize};
use web_time::Instant;

use crate::engine::serde_helpers::instant_now;

pub const DEFAULT_TRAVEL_SPEED: f32 = 30.0;
#[derive(PartialEq, Serialize, Deserialize)]
pub struct Laser {
    pub initial_z: f32,
    #[serde(skip, default = "instant_now")]
    pub fired_at: Instant,
    pub travel_speed: f32,
}
