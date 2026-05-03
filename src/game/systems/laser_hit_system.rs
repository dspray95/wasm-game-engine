use std::collections::HashSet;

use crate::{
    engine::{
        ecs::{
            entity::Entity, events::collision_event::CollisionEvent, system::SystemContext,
            systems::collision_system::filter_collision_pairs, world::World,
        },
        events::events::Events,
    },
    game::{
        components::{enemy::Enemy, laser::Laser},
        resources::{enemy_resources::EnemySpawnManager, laser_resources::LaserManager},
    },
};

pub fn laser_hit_system(world: &mut World, _system_context: &mut SystemContext) {
    let collision_events: Vec<(Entity, Entity)> = world
        .get_resource::<Events<CollisionEvent>>()
        .unwrap()
        .read()
        .map(|event| (event.a, event.b))
        .collect();

    let hits = filter_collision_pairs::<Laser, Enemy>(world, &collision_events);

    let lasers_to_despawn: HashSet<Entity> = hits.iter().map(|(laser, _)| *laser).collect();
    let enemies_to_despawn: HashSet<Entity> = hits.iter().map(|(_, enemy)| *enemy).collect();

    if let Some(laser_manager) = world.get_resource_mut::<LaserManager>() {
        laser_manager
            .alive_lasers
            .retain(|e| !lasers_to_despawn.contains(e));
    }

    if let Some(enemy_manager) = world.get_resource_mut::<EnemySpawnManager>() {
        enemy_manager
            .enemy_entities
            .retain(|e| !enemies_to_despawn.contains(e));
    }

    for entity in lasers_to_despawn.into_iter().chain(enemies_to_despawn) {
        world.despawn(entity);
    }
}
