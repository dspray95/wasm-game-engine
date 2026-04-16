use crate::engine::ecs::{
    components::{ transform::Transform, velocity::Velocity },
    system::SystemContext,
    world::World,
};

/// Applies each entity's Velocity to its Transform, then resets Velocity to zero.
/// Systems write to Velocity additively each frame — releasing an input simply
/// means no write happens, so the velocity stays at zero.
pub fn velocity_system(world: &mut World, system_context: &mut SystemContext) {
    let dt = system_context.delta_time;
    for (transform, velocity) in world.query_iter::<(&mut Transform, &mut Velocity)>() {
        transform.position.x += velocity.x * dt;
        transform.position.y += velocity.y * dt;
        transform.position.z += velocity.z * dt;
        velocity.x = 0.0;
        velocity.y = 0.0;
        velocity.z = 0.0;
    }
}
