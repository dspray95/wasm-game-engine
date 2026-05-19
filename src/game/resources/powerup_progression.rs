use crate::{
    engine::ecs::{entity::Entity, world::World},
    game::components::{
        double_fire_rate::DoubleFireRate, hyperdrive::Hyperdrive, shield::Shield,
    },
};

/// Identifies which powerup is next in the chain. Stack-derived: we look at
/// which powerups are *missing* from the player and grant the lowest-tier
/// missing one. If all three layers are present, the next pickup is the
/// chain-capping Bomb.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PowerUpKind {
    Shield,
    Laser,
    Hyperdrive,
    Bomb,
}

/// Look at the player's currently-active powerup components and decide what
/// the next pickup should grant.
///
/// Special-case: once Hyperdrive is active, the next pickup is always Bomb,
/// even if the player has since lost their Shield. The reasoning is that
/// rebuilding from rung 0 after a single shield-loss late-game would feel
/// punitive — once you've climbed to Hyperdrive, the bomb stays armed.
///
/// Otherwise: first missing slot in [Shield, Laser, Hyperdrive] order —
/// keeps the chain "wanting" to fill itself bottom-up.
pub fn next_powerup_kind(world: &World, player_entity: Entity) -> PowerUpKind {
    if world.get_component::<Hyperdrive>(player_entity).is_some() {
        return PowerUpKind::Bomb;
    }
    if world.get_component::<Shield>(player_entity).is_none() {
        PowerUpKind::Shield
    } else if world
        .get_component::<DoubleFireRate>(player_entity)
        .is_none()
    {
        PowerUpKind::Laser
    } else {
        PowerUpKind::Hyperdrive
    }
}
