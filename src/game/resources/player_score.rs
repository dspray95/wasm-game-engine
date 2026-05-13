pub struct PlayerScore {
    pub score: i32,
    pub increment: i32,
    pub multiplier: i32,
    pub increment_interval_seconds: f32,
    pub time_since_last_increment: f32,
}

impl PlayerScore {
    pub fn new() -> Self {
        Self {
            score: 0,
            increment: 1,
            multiplier: 1,
            increment_interval_seconds: 0.1,
            time_since_last_increment: 0.0,
        }
    }
}
