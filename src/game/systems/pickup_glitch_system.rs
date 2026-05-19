use crate::{
    engine::ecs::{
        components::{renderable::Renderable, transform::Transform},
        system::SystemContext,
        world::World,
    },
    game::components::{
        glitch_vfx::GlitchVfx, hyperdrive::Hyperdrive,
        hyperdrive_pickup_glitch::HyperdrivePickupGlitch, pickup::Pickup, player::Player,
    },
};

const PICKUP_GLITCH_MAX_OFFSET: f32 = 0.18;

/// While the player has Hyperdrive active, every Pickup wears the chromatic
/// chromatic-aberration treatment: base model swapped to the pure-white
/// variant, a cyan glitch sibling parented underneath. When Hyperdrive ends,
/// everything reverts.
///
/// State lives on `HyperdrivePickupGlitch` markers on the pickups themselves
/// so each pickup can independently track its base model + sibling entity
/// for cleanup. This system runs once per frame and reconciles the set:
/// any mismatch between "is player hyperdriving?" and "is pickup glitched?"
/// gets corrected.
pub fn pickup_glitch_system(world: &mut World, system_context: &mut SystemContext) {
    let hyperdrive_active = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| world.get_component_by_id::<Hyperdrive>(id).is_some())
        .unwrap_or(false);

    // Snapshot the pickup id list now so we can drop the World borrow before
    // mutating components / spawning new entities for reconciliation.
    let pickup_entity_ids: Vec<u32> = world
        .iter_component::<Pickup>()
        .map(|(id, _)| id)
        .collect();

    let Some(asset_server) = system_context.asset_server.as_deref() else {
        return;
    };
    let white_model_id = asset_server.get_model_id("pickup_white");
    let cyan_model_id = asset_server.get_model_id("pickup_glitch_cyan");

    for pickup_id in pickup_entity_ids {
        let already_glitched = world
            .get_component_by_id::<HyperdrivePickupGlitch>(pickup_id)
            .is_some();

        let Some(pickup_entity) = system_context.entity_allocator.lookup(pickup_id) else {
            continue;
        };

        match (hyperdrive_active, already_glitched) {
            (true, false) => {
                // Capture the current model_id so we can restore on revert.
                let base_model_id = match world
                    .get_component_by_id::<Renderable>(pickup_id)
                    .map(|r| r.model_id)
                {
                    Some(id) => id,
                    None => continue,
                };

                // Swap the pickup's base renderable to the pure-white variant.
                system_context
                    .commands
                    .update_component::<Renderable, _>(pickup_entity, move |r| {
                        r.model_id = white_model_id;
                    });

                // Spawn a cyan chromatic-aberration sibling, parented so it
                // follows the pickup's rotation + position automatically.
                let cyan_sibling = system_context
                    .commands
                    .spawn(system_context.entity_allocator)
                    .with(Renderable::new(cyan_model_id))
                    .with(Transform::new())
                    .with(GlitchVfx {
                        max_offset: PICKUP_GLITCH_MAX_OFFSET,
                        flash: None,
                    })
                    .as_child_of(pickup_entity)
                    .build();

                system_context.commands.add_component(
                    pickup_entity,
                    HyperdrivePickupGlitch {
                        base_model_id,
                        cyan_sibling,
                    },
                );
            }
            (false, true) => {
                let Some((base_model_id, cyan_sibling)) = world
                    .get_component_by_id::<HyperdrivePickupGlitch>(pickup_id)
                    .map(|g| (g.base_model_id, g.cyan_sibling))
                else {
                    continue;
                };

                system_context
                    .commands
                    .update_component::<Renderable, _>(pickup_entity, move |r| {
                        r.model_id = base_model_id;
                    });
                system_context.commands.despawn(cyan_sibling);
                system_context
                    .commands
                    .remove_component::<HyperdrivePickupGlitch>(pickup_entity);
            }
            _ => {}
        }
    }

}
