use crate::engine::ecs::entity::Entity;

/// Hyperdrive powerup: simultaneously boosts player z-speed AND doubles
/// score multiplier. The speed boost gives the score multiplier a *reason* —
/// you're surviving at higher risk, so the score gain is earned rather than
/// passive arithmetic.
pub struct Hyperdrive {
    pub time_remaining: f32,
    /// Glitch-VFX child entities parented to the player while active.
    /// Despawned explicitly when the powerup expires (the parent cascade
    /// covers player death).
    pub glitch_visuals: Vec<Entity>,
}
