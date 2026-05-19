/// 2-second wind-up between grabbing the bomb pickup and the actual nuke.
/// While present on the player: shake escalates BIG → SUPER, player is
/// invulnerable (the `Invulnerable` component is attached separately at
/// grab time so the flash mechanic re-uses existing code), and further
/// pickup grabs are ignored.
pub struct BombCharging {
    pub time_remaining: f32,
    pub total_seconds: f32,
}
