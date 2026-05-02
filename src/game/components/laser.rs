use web_time::Instant;

pub const DEFAULT_TRAVEL_SPEED: f32 = 30.0;

#[derive(PartialEq)]
pub struct Laser {
    pub initial_z: f32,
    pub fired_at: Instant,
    pub travel_speed: f32,
}
