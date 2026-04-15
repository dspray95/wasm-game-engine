use crate::engine::ecs::{
    components::{transform::Transform, velocity::Velocity},
    system::SystemContext,
    world::World,
};

/// Applies each entity's Velocity to its Transform position every frame.
/// Run this after all systems that write velocity, before render_sync_system.
pub fn velocity_system(world: &mut World, system_context: &mut SystemContext) {
    let dt = system_context.delta_time;
    for (transform, velocity) in world.query_iter::<(&mut Transform, &Velocity)>() {
        transform.position.x += velocity.x * dt;
        transform.position.y += velocity.y * dt;
        transform.position.z += velocity.z * dt;
    }
}
