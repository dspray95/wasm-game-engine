/// Marks an entity as a glitch-VFX child. Its local `Transform.position` is
/// jittered each frame by `vfx_system` for the chromatic-aberration shoogle.
///
/// Lifetime is tied to the parent via the hierarchy's despawn cascade —
/// when the parent (player or laser) dies, the glitch children go with it.
/// Player-attached glitches are also despawned when DoublePoints expires.
///
/// `flash` controls visibility scheduling:
/// - `None` → always visible, shoogled every frame (used for short-lived
///   laser glitches where you want a constant chromatic streak).
/// - `Some(FlashSchedule)` → toggle between flashing/idle phases with random
///   durations from the given ranges. Used for the player ship so the
///   glitch reads as discrete flashes rather than a constant blur.
pub struct GlitchVfx {
    pub max_offset: f32,
    pub flash: Option<FlashSchedule>,
}

pub struct FlashSchedule {
    pub min_idle_seconds: f32,
    pub max_idle_seconds: f32,
    pub min_flash_seconds: f32,
    pub max_flash_seconds: f32,
    pub phase_remaining: f32,
    pub is_flashing: bool,
}
