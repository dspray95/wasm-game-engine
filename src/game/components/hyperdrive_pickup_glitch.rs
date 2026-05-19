use crate::engine::ecs::entity::Entity;

/// Marker attached to a `Pickup` while the player has Hyperdrive active. It
/// remembers the original pickup `model_id` (so we can revert when
/// Hyperdrive ends) and the cyan chromatic-aberration sibling entity (so we
/// can despawn it on revert). `pickup_glitch_system` reconciles state each
/// frame: present when Hyperdrive active, removed when not.
pub struct HyperdrivePickupGlitch {
    pub base_model_id: usize,
    pub cyan_sibling: Entity,
}
