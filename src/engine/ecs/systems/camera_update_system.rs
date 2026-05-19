use crate::engine::ecs::{
    components::{camera::camera::Camera, world_transform::WorldTransform},
    resources::camera::ActiveCamera,
    system::SystemContext,
    world::World,
};

/// Always sync CPU state → GPU buffer every frame. The camera may have been moved
/// by other game systems, so the buffer must stay current.
pub fn camera_update_system(world: &mut World, system_context: &mut SystemContext) {
    let Some(active_camera) = world.get_resource::<ActiveCamera>() else {
        return;
    };
    let active_camera_entity = active_camera.0;

    // We need to keep the cameras view projection and world position on the GPU in sync with
    // its entity's WorldTransform position (composed by hierarchy_system; for an unparented
    // camera this just mirrors the local Transform).
    let Some(world_transform) = world.get_component::<WorldTransform>(active_camera_entity) else {
        return;
    };
    let position = world_transform.position;

    let Some(camera) = world.get_component_mut::<Camera>(active_camera_entity) else {
        return;
    };

    let shaken_position = position + camera.shake_offset;
    camera.update_view_projeciton(shaken_position);
    camera.update_position(shaken_position);

    system_context.queue
        .unwrap()
        .write_buffer(
            &camera.render_pass_data.buffer,
            0,
            bytemuck::cast_slice(&[camera.render_pass_data.uniform_buffer])
        );
}
