use cgmath::Vector3;
use rand::Rng;

use crate::{
    engine::ecs::{
        components::{renderable::Renderable, transform::Transform},
        system::SystemContext,
        world::World,
    },
    game::components::glitch_vfx::GlitchVfx,
};

/// Per-frame jitter of every `GlitchVfx` entity's local `Transform.position`,
/// producing the chromatic-aberration shoogle. Each axis gets a fresh
/// `[-max_offset, max_offset]` value every frame.
///
/// If the entity carries a `FlashSchedule`, visibility is toggled between
/// short flash bursts and longer idle gaps; only the flashing phase produces
/// jitter + visible renderable. Without a schedule, the glitch is always on.
///
/// Because glitch children are parented to their target (player or laser),
/// the hierarchy_system composes the jittered local offset onto the parent's
/// world transform — so the shoogle reads correctly even as the parent moves.
pub fn vfx_system(world: &mut World, _system_context: &mut SystemContext) {
    let delta_time = _system_context.delta_time;
    let mut rng = rand::rng();

    // Snapshot every glitch entity into a plain Vec so we can drop the borrow
    // before mutating the per-entity components below.
    let snapshots: Vec<GlitchSnapshot> = world
        .iter_component::<GlitchVfx>()
        .map(|(entity_id, glitch)| GlitchSnapshot {
            entity_id,
            max_offset: glitch.max_offset,
            flash_state: glitch.flash.as_ref().map(|f| FlashSnapshot {
                phase_remaining: f.phase_remaining,
                is_flashing: f.is_flashing,
                min_idle: f.min_idle_seconds,
                max_idle: f.max_idle_seconds,
                min_flash: f.min_flash_seconds,
                max_flash: f.max_flash_seconds,
            }),
        })
        .collect();

    for snap in snapshots {
        let (should_jitter, visible, new_flash) = match snap.flash_state {
            None => (true, true, None),
            Some(state) => {
                let next_remaining = state.phase_remaining - delta_time;
                if next_remaining > 0.0 {
                    (state.is_flashing, state.is_flashing, Some((next_remaining, state.is_flashing)))
                } else {
                    // Phase transition: flip is_flashing and roll new duration
                    // from the appropriate range.
                    let new_is_flashing = !state.is_flashing;
                    let new_remaining = if new_is_flashing {
                        rng.random_range(state.min_flash..state.max_flash)
                    } else {
                        rng.random_range(state.min_idle..state.max_idle)
                    };
                    (new_is_flashing, new_is_flashing, Some((new_remaining, new_is_flashing)))
                }
            }
        };

        let jitter = if should_jitter {
            Vector3::new(
                rng.random_range(-snap.max_offset..snap.max_offset),
                rng.random_range(-snap.max_offset..snap.max_offset),
                rng.random_range(-snap.max_offset..snap.max_offset),
            )
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };

        if let Some(transform) = world.get_component_mut_by_id::<Transform>(snap.entity_id) {
            transform.position = jitter;
        }
        if let Some(renderable) = world.get_component_mut_by_id::<Renderable>(snap.entity_id) {
            renderable.visible = visible;
        }
        if let Some((remaining, is_flashing)) = new_flash {
            if let Some(glitch) = world.get_component_mut_by_id::<GlitchVfx>(snap.entity_id) {
                if let Some(flash) = glitch.flash.as_mut() {
                    flash.phase_remaining = remaining;
                    flash.is_flashing = is_flashing;
                }
            }
        }
    }
}

struct GlitchSnapshot {
    entity_id: u32,
    max_offset: f32,
    flash_state: Option<FlashSnapshot>,
}

struct FlashSnapshot {
    phase_remaining: f32,
    is_flashing: bool,
    min_idle: f32,
    max_idle: f32,
    min_flash: f32,
    max_flash: f32,
}
