pub const GLITCH_DURATION_SECONDS: f32 = 1.0;
pub const SHAKE_SMALL_DURATION_SECONDS: f32 = 0.15;
pub const SHAKE_BIG_DURATION_SECONDS: f32 = 0.5;
pub const SHAKE_SMALL_MAGNITUDE: f32 = 0.01;
pub const SHAKE_BIG_MAGNITUDE: f32 = 0.04;

pub struct ScreenEffects {
    pub glitch_timer: f32,
    pub shake_timer: f32,
    pub shake_duration: f32,
    pub shake_magnitude: f32,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            glitch_timer: 0.0,
            shake_timer: 0.0,
            shake_duration: SHAKE_SMALL_DURATION_SECONDS,
            shake_magnitude: SHAKE_SMALL_MAGNITUDE,
        }
    }

    pub fn trigger_kill_effect(&mut self) {
        self.glitch_timer = GLITCH_DURATION_SECONDS;
        self.trigger_shake(SHAKE_SMALL_DURATION_SECONDS, SHAKE_SMALL_MAGNITUDE);
    }

    pub fn trigger_damage_effect(&mut self) {
        self.trigger_shake(SHAKE_BIG_DURATION_SECONDS, SHAKE_BIG_MAGNITUDE);
    }

    /// Replace the active shake with `(duration, magnitude)` only if the new
    /// shake is stronger than what's currently active. Prevents a tiny kill
    /// shake from cutting short an in-progress damage shake.
    fn trigger_shake(&mut self, duration: f32, magnitude: f32) {
        if magnitude >= self.shake_magnitude || self.shake_timer <= 0.0 {
            self.shake_timer = duration;
            self.shake_duration = duration;
            self.shake_magnitude = magnitude;
        }
    }

    pub fn glitch_intensity(&self) -> f32 {
        (self.glitch_timer / GLITCH_DURATION_SECONDS).clamp(0.0, 1.0)
    }

    pub fn shake_intensity(&self) -> f32 {
        if self.shake_duration <= 0.0 {
            return 0.0;
        }
        (self.shake_timer / self.shake_duration).clamp(0.0, 1.0)
    }
}
