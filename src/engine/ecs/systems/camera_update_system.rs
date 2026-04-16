use crate::engine::{ camera::camera::Camera, ecs::{ system::SystemContext, world::World } };

/// Always sync CPU state → GPU buffer every frame. The camera may have been moved
/// by other game systems, so the buffer must stay current.
pub fn camera_update_system(world: &mut World, system_context: &mut SystemContext) {
    let camera: &mut Camera = world.get_resource_mut::<Camera>().unwrap();

    camera.update_view_projeciton();
    camera.update_position();
    system_context.queue
        .unwrap()
        .write_buffer(
            &camera.render_pass_data.buffer,
            0,
            bytemuck::cast_slice(&[camera.render_pass_data.uniform_buffer])
        );
}
