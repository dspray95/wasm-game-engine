use cgmath::Vector3;
use serde::{Deserialize, Serialize};
use web_time::Instant;

use crate::engine::serde_helpers::instant_now;

const DEFUALT_LIFETIME_SECONDS: f32 = 0.15;
const DEFAUTL_TIME_BETWEEN_ROTATIONS: f32 = 0.05;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Explosion {
    #[serde(skip, default = "instant_now")]
    pub created_at: Instant,
    pub lifetime_seconds: f32,
    pub time_between_rotations: f32,
    #[serde(skip, default = "instant_now")]
    pub last_rotated_at: Instant,
    #[serde(default = "zero_velocity")]
    pub carry_velocity: Vector3<f32>,
}

fn zero_velocity() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}

impl Explosion {
    pub fn new() -> Self {
        Self {
            created_at: Instant::now(),
            lifetime_seconds: DEFUALT_LIFETIME_SECONDS,
            time_between_rotations: DEFAUTL_TIME_BETWEEN_ROTATIONS,
            last_rotated_at: Instant::now(),
            carry_velocity: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn with_velocity(mut self, velocity: Vector3<f32>) -> Self {
        self.carry_velocity = velocity;
        self
    }
}
