use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::{
        components::{
            double_fire_rate::DoubleFireRate, hyperdrive::Hyperdrive, player::Player,
        },
        resources::player_score::PlayerScore,
    },
};

pub fn powerup_lifecycle_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(player_entity) = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next()
    else {
        return;
    };

    let delta_time = system_context.delta_time;

    // Hyperdrive: tick timer; on expiry, reset PlayerScore.multiplier so
    // passive + kill scoring return to 1× and despawn the glitch-VFX children
    // we attached on grant. The missing slot makes the next pickup grant
    // Hyperdrive again (chain backfills from the bottom missing layer).
    if let Some((new_time, glitch_visuals)) = world
        .get_component::<Hyperdrive>(player_entity)
        .map(|p| (p.time_remaining - delta_time, p.glitch_visuals.clone()))
    {
        if new_time <= 0.0 {
            system_context
                .commands
                .remove_component::<Hyperdrive>(player_entity);
            system_context
                .commands
                .update_resource::<PlayerScore, _>(|score| {
                    score.multiplier = 1;
                });
            for visual in glitch_visuals {
                system_context.commands.despawn(visual);
            }
        } else {
            system_context
                .commands
                .update_component::<Hyperdrive, _>(player_entity, move |p| {
                    p.time_remaining = new_time;
                });
        }
    }

    // Double fire-rate: tick timer; on expiry, just remove the component.
    if let Some(new_time) = world
        .get_component::<DoubleFireRate>(player_entity)
        .map(|p| p.time_remaining - delta_time)
    {
        if new_time <= 0.0 {
            system_context
                .commands
                .remove_component::<DoubleFireRate>(player_entity);
        } else {
            system_context
                .commands
                .update_component::<DoubleFireRate, _>(player_entity, move |p| {
                    p.time_remaining = new_time;
                });
        }
    }
}
