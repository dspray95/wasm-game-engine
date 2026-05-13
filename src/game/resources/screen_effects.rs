pub const GLITCH_DURATION_SECONDS: f32 = 1.0;
pub const SHAKE_DURATION_SECONDS: f32 = 0.15;
pub const SHAKE_SMALL_MAGNITUDE: f32 = 0.01;

pub struct ScreenEffects {
    pub glitch_timer: f32,
    pub shake_timer: f32,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            glitch_timer: 0.0,
            shake_timer: 0.0,
        }
    }

    pub fn trigger_kill_effect(&mut self) {
        self.glitch_timer = GLITCH_DURATION_SECONDS;
        self.shake_timer = SHAKE_DURATION_SECONDS;
    }

    pub fn glitch_intensity(&self) -> f32 {
        (self.glitch_timer / GLITCH_DURATION_SECONDS).clamp(0.0, 1.0)
    }

    pub fn shake_intensity(&self) -> f32 {
        (self.shake_timer / SHAKE_DURATION_SECONDS).clamp(0.0, 1.0)
    }
}
