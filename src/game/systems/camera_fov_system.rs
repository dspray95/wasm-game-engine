use cgmath::{Deg, Rad};

use crate::{
    engine::ecs::{
        components::camera::camera::Camera, resources::camera::ActiveCamera,
        system::SystemContext, world::World,
    },
    game::components::{hyperdrive::Hyperdrive, player::Player},
};

/// Default FOV (matches engine default in DEFAULT_FOV) — used as the resting
/// target when no FOV-affecting powerup is active.
const DEFAULT_FOV_DEG: f32 = 45.0;
/// Widened FOV while Hyperdrive is active. ~15° widening reads as a tangible
/// "world is opening up" feel without distorting the silhouettes badly.
const HYPERDRIVE_FOV_DEG: f32 = 62.0;
/// Lerp rate (per second) — how fast we move from current → target FOV. 5.0
/// gives a ~0.2s transition; quick enough to feel snappy, slow enough that
/// you don't get jarred when grabbing the pickup or when it expires.
const FOV_LERP_RATE: f32 = 5.0;

pub fn camera_fov_system(world: &mut World, system_context: &mut SystemContext) {
    let hyperdrive_active = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| world.get_component_by_id::<Hyperdrive>(id).is_some())
        .unwrap_or(false);

    let target_fov: Rad<f32> = if hyperdrive_active {
        Deg(HYPERDRIVE_FOV_DEG).into()
    } else {
        Deg(DEFAULT_FOV_DEG).into()
    };

    let Some(camera_entity) = world.get_resource::<ActiveCamera>().map(|ac| ac.0) else {
        return;
    };
    let Some(camera) = world.get_component_mut::<Camera>(camera_entity) else {
        return;
    };

    let current = camera.projection.fov_y();
    // Exponential approach toward target — frame-rate independent because
    // 1 - exp(-rate * dt) is the proportion of the remaining distance we
    // close this frame. Equivalent to a critically-damped lerp in feel.
    let t = 1.0 - (-FOV_LERP_RATE * system_context.delta_time).exp();
    let new_fov = Rad(current.0 + (target_fov.0 - current.0) * t);
    camera.projection.set_fov_y(new_fov);
}
