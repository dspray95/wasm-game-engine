use cgmath::{Quaternion, Rad, Rotation3};

use crate::{
    engine::ecs::{
        components::{renderable::Renderable, transform::Transform},
        system::SystemContext,
        world::World,
    },
    game::components::{player::Player, shield::Shield},
};

const SHIELD_ROTATION_RADIANS_PER_SECOND: f32 = 3.5;
const SHIELD_BLINK_THRESHOLD_SECONDS: f32 = 1.5;
const SHIELD_FLASH_PHASE_SECONDS: f32 = 0.1;

pub fn shield_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    let Some(shield_snapshot) = world.get_component::<Shield>(player_entity).map(|s| {
        (
            s.time_remaining - system_context.delta_time,
            s.visual_entity,
            s.rotation_axis,
            s.flash_phase_timer - system_context.delta_time,
        )
    }) else {
        return;
    };

    let (new_time_remaining, visual_entity, rotation_axis, mut new_flash_phase_timer) =
        shield_snapshot;

    if new_time_remaining <= 0.0 {
        // Explicit despawn here is the mechanism; the parent's despawn cascade
        // would also catch this if the player died, but expiring on time
        // doesn't go through that path.
        system_context
            .commands
            .remove_component::<Shield>(player_entity);
        system_context.commands.despawn(visual_entity);
        // Stack model: shield expired naturally → no other state to update;
        // the missing slot makes the next pickup grant Shield again.
        return;
    }

    // Spin around the shield's random axis. We write into the visual's local
    // Transform.rotation; hierarchy_system composes that with the player's
    // (identity) rotation each frame, so the spin reads correctly in world
    // space.
    let spin = Quaternion::from_axis_angle(
        rotation_axis,
        Rad(SHIELD_ROTATION_RADIANS_PER_SECOND * system_context.delta_time),
    );
    system_context
        .commands
        .update_component::<Transform, _>(visual_entity, move |t| {
            t.rotation = spin * t.rotation;
        });

    // Blink near expiry: toggle Renderable.visible every flash phase. Outside
    // the blink window keep visible forced true so it can't get stuck off.
    let blinking = new_time_remaining < SHIELD_BLINK_THRESHOLD_SECONDS;
    let mut toggle_visible = false;
    if blinking && new_flash_phase_timer <= 0.0 {
        toggle_visible = true;
        new_flash_phase_timer = SHIELD_FLASH_PHASE_SECONDS;
    }

    system_context
        .commands
        .update_component::<Shield, _>(player_entity, move |s| {
            s.time_remaining = new_time_remaining;
            s.flash_phase_timer = new_flash_phase_timer;
        });

    if blinking {
        if toggle_visible {
            system_context
                .commands
                .update_component::<Renderable, _>(visual_entity, |r| {
                    r.visible = !r.visible;
                });
        }
    } else {
        system_context
            .commands
            .update_component::<Renderable, _>(visual_entity, |r| {
                r.visible = true;
            });
    }
}
