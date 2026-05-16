use cgmath::Vector3;

use crate::engine::ecs::{
    components::{camera::camera::Camera, transform::Transform},
    resources::camera::ActiveCamera,
    world::World,
};

/// Project a world-space point to egui screen-pixel coordinates using the
/// `ActiveCamera`.
///
/// Returns `None` if any required piece is missing (no active camera, the
/// camera entity is missing its `Transform` or `Camera` component) or if the
/// point is behind the camera plane.
///
/// `screen_rect` should be the egui context's [`screen_rect`](egui::Context::screen_rect),
/// passed in so this helper does not depend on the live `Context` and so the
/// caller can substitute a sub-region (split-screen, picture-in-picture) when
/// needed.
pub fn world_to_screen(
    world: &World,
    world_position: Vector3<f32>,
    screen_rect: egui::Rect,
) -> Option<egui::Pos2> {
    let camera_entity = world.get_resource::<ActiveCamera>()?.0;
    let camera_position = world
        .get_component_by_id::<Transform>(camera_entity.id)?
        .position;
    let camera = world.get_component_by_id::<Camera>(camera_entity.id)?;

    let (ndc_x, ndc_y) = camera.world_to_screen(world_position, camera_position)?;

    // NDC [-1, 1] → screen pixels. y is flipped because NDC y points up while
    // egui screen y grows downward.
    let screen_x = screen_rect.left() + (ndc_x * 0.5 + 0.5) * screen_rect.width();
    let screen_y = screen_rect.top() + (1.0 - (ndc_y * 0.5 + 0.5)) * screen_rect.height();
    Some(egui::pos2(screen_x, screen_y))
}
