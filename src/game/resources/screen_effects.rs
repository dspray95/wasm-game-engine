pub const GLITCH_DURATION_SECONDS: f32 = 1.0;
pub const SHAKE_SMALL_DURATION_SECONDS: f32 = 0.15;
pub const SHAKE_BIG_DURATION_SECONDS: f32 = 0.5;
pub const SHAKE_SMALL_MAGNITUDE: f32 = 0.01;
pub const SHAKE_BIG_MAGNITUDE: f32 = 0.04;
/// Peak magnitude the bomb's wind-up ramps toward — meaningfully harder than
/// damage shake so the player feels the moment land.
pub const SHAKE_SUPER_MAGNITUDE: f32 = 0.12;

pub struct ScreenEffects {
    pub glitch_timer: f32,
    pub shake_timer: f32,
    pub shake_duration: f32,
    pub shake_magnitude: f32,
    /// Full-screen white flash overlay. Ticks down each frame; the UI panel
    /// renders alpha = `white_flash_timer / white_flash_duration`. Used by
    /// the bomb detonation.
    pub white_flash_timer: f32,
    pub white_flash_duration: f32,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            glitch_timer: 0.0,
            shake_timer: 0.0,
            shake_duration: SHAKE_SMALL_DURATION_SECONDS,
            shake_magnitude: SHAKE_SMALL_MAGNITUDE,
            white_flash_timer: 0.0,
            white_flash_duration: 0.0,
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
    pub fn trigger_shake(&mut self, duration: f32, magnitude: f32) {
        if magnitude >= self.shake_magnitude || self.shake_timer <= 0.0 {
            self.shake_timer = duration;
            self.shake_duration = duration;
            self.shake_magnitude = magnitude;
        }
    }

    pub fn trigger_white_flash(&mut self, duration: f32) {
        self.white_flash_timer = duration;
        self.white_flash_duration = duration;
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

    pub fn white_flash_intensity(&self) -> f32 {
        if self.white_flash_duration <= 0.0 {
            return 0.0;
        }
        (self.white_flash_timer / self.white_flash_duration).clamp(0.0, 1.0)
    }
}
