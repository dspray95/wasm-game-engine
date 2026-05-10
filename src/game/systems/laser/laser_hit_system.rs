use std::collections::HashSet;

use crate::{
    engine::{
        ecs::{
            components::transform::Transform, entity::Entity,
            events::collision_event::CollisionEvent, system::SystemContext,
            systems::collision_system::filter_collision_pairs, world::World,
        },
        events::events::Events,
    },
    game::{
        components::{enemy::Enemy, laser::Laser},
        events::{
            enemy_killed_event::EnemyKilledEvent,
            score_event::{ScoreEvent, ScoreType},
        },
        resources::{enemy_resources::EnemySpawnManager, laser_resources::LaserManager},
    },
};

pub fn laser_hit_system(world: &mut World, system_context: &mut SystemContext) {
    let collision_events: Vec<(Entity, Entity)> = world
        .get_resource::<Events<CollisionEvent>>()
        .unwrap()
        .read()
        .map(|event| (event.a, event.b))
        .collect();

    let hits = filter_collision_pairs::<Laser, Enemy>(world, &collision_events);

    let lasers_to_despawn: HashSet<Entity> = hits.iter().map(|(laser, _)| *laser).collect();
    let enemies_to_despawn: HashSet<Entity> = hits.iter().map(|(_, enemy)| *enemy).collect();

    let enemy_killed_events: Vec<EnemyKilledEvent> = enemies_to_despawn
        .iter()
        .filter_map(|enemy| {
            world
                .get_component_by_id::<Transform>(enemy.id)
                .map(|t| EnemyKilledEvent { origin: t.position })
        })
        .collect();

    let cmd = &mut system_context.commands;

    {
        let lasers = lasers_to_despawn.clone();
        cmd.update_resource::<LaserManager, _>(move |m| {
            m.alive_lasers.retain(|e| !lasers.contains(e));
        });
    }
    {
        let enemies = enemies_to_despawn.clone();
        cmd.update_resource::<EnemySpawnManager, _>(move |m| {
            m.enemy_entities.retain(|e| !enemies.contains(e));
        });
    }

    for entity in lasers_to_despawn.into_iter().chain(enemies_to_despawn) {
        cmd.despawn(entity);
    }

    for _ in 0..enemy_killed_events.len() {
        cmd.send_event(ScoreEvent {
            score_type: ScoreType::EnemyKilled,
        });
    }
    for event in enemy_killed_events {
        cmd.send_event(event);
    }
}
