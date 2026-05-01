use cgmath::{ One, Quaternion, Vector3 };

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            components::{
                collider::{ Collider, ColliderShape },
                renderable::Renderable,
                transform::Transform,
            },
            system::SystemContext,
            world::World,
        },
    },
    game::{
        components::{ hover_state::{ HoverDirection, HoverState }, player::Player },
        resources::enemy_resources::EnemySpawnManager,
    },
};

pub fn enemy_spawn_system(world: &mut World, system_context: &mut SystemContext) {
    let player_position = world
        .iter_component::<Player>()
        .next()
        .and_then(|(entity_id, _)| world.get_component_by_id::<Transform>(entity_id))
        .map(|transform| transform.position);

    let Some(player_position) = player_position else {
        return;
    };

    let (spawn_enemy_at, enemy_spawn_scale): (Option<Vector3<f32>>, Option<Vector3<f32>>) = {
        let Some(enemy_spawn_manager) = world.get_resource_mut::<EnemySpawnManager>() else {
            return;
        };

        if
            player_position.z >
            enemy_spawn_manager.last_z_pos_spawned_at + enemy_spawn_manager.z_gap_between_spanws
        {
            let spawn_at_z = player_position.z + enemy_spawn_manager.z_gap_between_spanws;
            enemy_spawn_manager.last_z_pos_spawned_at = spawn_at_z;
            (
                Some(Vector3 {
                    x: enemy_spawn_manager.canyon_center_x,
                    y: enemy_spawn_manager.enemy_spawn_elevation,
                    z: spawn_at_z,
                }),
                Some(enemy_spawn_manager.enemy_spawn_scale),
            )
        } else {
            (None, None)
        }
    };

    // spawn_enemy needs another mut borrow of world, hence calling it here
    if spawn_enemy_at.is_some() && enemy_spawn_scale.is_some() {
        spawn_enemy(
            world,
            system_context.asset_server.as_deref().unwrap(),
            spawn_enemy_at.unwrap(),
            enemy_spawn_scale.unwrap()
        );
    }
}

fn spawn_enemy(
    world: &mut World,
    asset_server: &AssetServer,
    position: Vector3<f32>,
    scale: Vector3<f32>
) {
    log::info!("Spawnning enemy at z: {:?}", position);
    let starfigher_model_id = asset_server.get_model_id("starfighter");
    world
        .spawn()
        .with(Renderable { model_id: starfigher_model_id })
        .with(Collider {
            shape: ColliderShape::AABB {
                offset: Vector3::new(0.0, 0.0, -0.3),
                half_extents: Vector3::new(1.0, 0.5, 1.5),
            },
        })
        .with(Transform {
            position,
            scale,
            rotation: Quaternion::one(),
        })
        .with(HoverState { direction: HoverDirection::Down, upper_limit: -0.9, lower_limit: -0.99 })
        .build();
}
