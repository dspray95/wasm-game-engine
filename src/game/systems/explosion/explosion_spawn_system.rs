use cgmath::{One, Quaternion, Vector3};

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            components::{renderable::Renderable, transform::Transform},
            system::SystemContext,
            world::World,
        },
        events::events::Events,
    },
    game::{components::explosion::Explosion, events::enemy_killed_event::EnemyKilledEvent},
};

pub fn explosion_spawn_system(world: &mut World, system_context: &mut SystemContext) {
    let enemy_killed_event_positions: Vec<Vector3<f32>> = world
        .get_resource::<Events<EnemyKilledEvent>>()
        .unwrap()
        .read()
        .map(|e| {
            log::info!("Read enemy killed event");
            e.origin
        })
        .collect();

    for origin in enemy_killed_event_positions {
        log::info!("spawning explosion");
        spawn_explosion(
            world,
            system_context.asset_server.as_deref().unwrap(),
            origin,
        );
    }
}

fn spawn_explosion(world: &mut World, asset_server: &AssetServer, position: Vector3<f32>) {
    world
        .spawn()
        .with(Explosion::new())
        .with(Transform {
            position,
            rotation: Quaternion::one(),
            scale: Vector3 {
                x: 0.15,
                y: 0.15,
                z: 0.15,
            },
        })
        .with(Renderable {
            model_id: asset_server.get_model_id("explosion"),
        })
        .build();
}
