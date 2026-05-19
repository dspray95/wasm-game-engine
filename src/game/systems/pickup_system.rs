use cgmath::{Quaternion, Rad, Rotation3, Vector3};

use crate::{
    engine::ecs::{
        components::{transform::Transform, world_transform::WorldTransform},
        entity::Entity,
        system::SystemContext,
        world::World,
    },
    game::components::{pickup::Pickup, player::Player},
};

const PICKUP_ROTATION_RADIANS_PER_SECOND: f32 = 2.5;
const PICKUP_DESPAWN_BEHIND_DISTANCE: f32 = 20.0;

pub fn pickup_system(world: &mut World, system_context: &mut SystemContext) {
    let player_z = world
        .iter_component::<Player>()
        .next()
        .and_then(|(id, _)| world.get_component_by_id::<WorldTransform>(id))
        .map(|t| t.position.z);

    let delta_time = system_context.delta_time;
    let spin =
        Quaternion::from_axis_angle(Vector3::unit_y(), Rad(PICKUP_ROTATION_RADIANS_PER_SECOND * delta_time));

    let mut entities_to_despawn: Vec<Entity> = Vec::new();
    let despawn_z = player_z.map(|z| z - PICKUP_DESPAWN_BEHIND_DISTANCE);

    for (transform, _) in world.query_iter_mut::<(&mut Transform, &Pickup)>() {
        transform.rotation = spin * transform.rotation;
    }

    if let Some(threshold_z) = despawn_z {
        for (entity_id, _) in world.iter_component::<Pickup>() {
            if let Some(transform) = world.get_component_by_id::<WorldTransform>(entity_id) {
                if transform.position.z < threshold_z {
                    if let Some(entity) = system_context.entity_allocator.lookup(entity_id) {
                        entities_to_despawn.push(entity);
                    }
                }
            }
        }
    }

    for entity in entities_to_despawn {
        system_context.commands.despawn(entity);
    }
}
