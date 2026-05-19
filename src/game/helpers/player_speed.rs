use crate::{
    engine::ecs::world::World,
    game::{
        components::{hyperdrive::Hyperdrive, player::Player},
        resources::{player_score::PlayerScore, player_speed_scaling::PlayerSpeedScaling},
    },
};

/// Multiplier applied to the player's z-speed while the Hyperdrive powerup
/// is active
pub const HYPERDRIVE_SPEED_MULTIPLIER: f32 = 1.5;

/// Single source of truth for the player's *current* effective z-speed.
/// Combines the score-driven base curve with any active speed-modifying
/// powerups (currently just Hyperdrive). Three systems read this:
/// `player_system` for movement + camera follow, `laser_system` for laser
/// velocity, `player_damage_system` for the speed at death.
pub fn effective_player_z_speed(world: &World) -> f32 {
    let score = world
        .get_resource::<PlayerScore>()
        .map(|s| s.score)
        .unwrap_or(0);
    let base = world
        .get_resource::<PlayerSpeedScaling>()
        .map(|scaling| scaling.z_speed.value(score))
        .unwrap_or(0.0);

    let hyperdrive_active = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| world.get_component_by_id::<Hyperdrive>(id).is_some())
        .unwrap_or(false);

    if hyperdrive_active {
        base * HYPERDRIVE_SPEED_MULTIPLIER
    } else {
        base
    }
}
