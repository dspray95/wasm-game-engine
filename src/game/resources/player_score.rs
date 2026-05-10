pub struct PlayerScore {
    score: i32,
    increment: i32,
    multiplier: i32,
}

impl PlayerScore {
    pub fn new() -> Self {
        Self {
            score: 0,
            increment: 10,
            multiplier: 1,
        }
    }
}
