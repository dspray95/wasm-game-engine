use crate::engine::{
    ecs::{
        components::{ camera::camera::Camera, transform::Transform },
        resources::camera::ActiveCamera,
        system::SystemContext,
        world::World,
    },
};

/// Always sync CPU state → GPU buffer every frame. The camera may have been moved
/// by other game systems, so the buffer must stay current.
pub fn camera_update_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(active_camera) = world.get_resource::<ActiveCamera>() else {
        return;
    };
    let active_camera_entity = active_camera.0;

    // We need to keep the cameras view projection and world position on the GPU in sync with
    // its entity's Transform position
    let Some(transform) = world.get_component::<Transform>(active_camera_entity) else {
        return;
    };
    let position = transform.position;

    let Some(camera) = world.get_component_mut::<Camera>(active_camera_entity) else {
        return;
    };

    camera.update_view_projeciton(position);
    camera.update_position(position);

    system_context.queue
        .unwrap()
        .write_buffer(
            &camera.render_pass_data.buffer,
            0,
            bytemuck::cast_slice(&[camera.render_pass_data.uniform_buffer])
        );
}
