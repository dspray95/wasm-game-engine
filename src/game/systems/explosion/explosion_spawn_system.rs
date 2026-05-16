use cgmath::{One, Quaternion, Vector3};

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            components::{renderable::Renderable, transform::Transform},
            entity::EntityAllocator,
            system::SystemContext,
            world::World,
        },
        events::events::Events,
    },
    game::{
        components::explosion::Explosion,
        events::{enemy_killed_event::EnemyKilledEvent, player_died_event::PlayerDiedEvent},
    },
};

pub fn explosion_spawn_system(world: &mut World, system_context: &mut SystemContext) {
    let mut requests: Vec<(Vector3<f32>, Vector3<f32>)> = world
        .get_resource::<Events<EnemyKilledEvent>>()
        .unwrap()
        .read()
        .map(|e| (e.origin, Vector3::new(0.0, 0.0, 0.0)))
        .collect();

    requests.extend(
        world
            .get_resource::<Events<PlayerDiedEvent>>()
            .unwrap()
            .read()
            .map(|e| (e.origin, e.velocity)),
    );

    for (origin, velocity) in requests {
        spawn_explosion(
            world,
            system_context.asset_server.as_deref().unwrap(),
            origin,
            velocity,
            system_context.entity_allocator,
        );
    }
}

fn spawn_explosion(
    world: &mut World,
    asset_server: &AssetServer,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    allocator: &mut EntityAllocator,
) {
    world
        .spawn(allocator)
        .with(Explosion::new().with_velocity(velocity))
        .with(Transform {
            position,
            rotation: Quaternion::one(),
            scale: Vector3 {
                x: 0.15,
                y: 0.15,
                z: 0.15,
            },
        })
        .with(Renderable::new(asset_server.get_model_id("explosion")))
        .build();
}
