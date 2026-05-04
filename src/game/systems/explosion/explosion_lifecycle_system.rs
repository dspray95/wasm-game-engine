use cgmath::{InnerSpace, Quaternion, Rad, Rotation3};
use rand::Rng;
use web_time::{Duration, Instant};

use crate::{
    engine::ecs::{components::transform::Transform, system::SystemContext, world::World},
    game::components::explosion::Explosion,
};

pub fn explosion_lifecycle_system(world: &mut World, _system_context: &mut SystemContext) {
    let now = Instant::now();

    let expired_explosions: Vec<u32> = world
        .iter_component::<Explosion>()
        .filter(|(_, explosion)| {
            let time_diff =
                explosion.created_at + Duration::from_secs_f32(explosion.lifetime_seconds);
            now > time_diff
        })
        .map(|(id, _)| id)
        .collect();

    for entity_id in expired_explosions {
        if let Some(entity) = world.get_entity(entity_id) {
            world.despawn(entity);
        }
    }

    let mut rng = rand::rng();
    for (explosion, transform) in world.query_iter::<(&mut Explosion, &mut Transform)>() {
        if now
            > explosion.last_rotated_at + Duration::from_secs_f32(explosion.time_between_rotations)
        {
            let axis = cgmath::Vector3::new(
                rng.random::<f32>() * 2.0 - 1.0,
                rng.random::<f32>() * 2.0 - 1.0,
                rng.random::<f32>() * 2.0 - 1.0,
            )
            .normalize();
            let angle = Rad(rng.random::<f32>() * std::f32::consts::TAU);
            transform.rotation = Quaternion::from_axis_angle(axis, angle);

            explosion.last_rotated_at = now;
        }
    }
}
