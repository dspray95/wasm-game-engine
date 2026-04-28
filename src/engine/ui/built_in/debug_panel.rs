use crate::engine::{ ecs::world::World, fps_counter::FpsCounter };

pub fn debug_panel(context: &egui::Context, world: &mut World) {
    let fps = world
        .get_resource::<FpsCounter>()
        .map(|f| f.get_fps())
        .unwrap_or(0.0);
    let n_entities = world.live_entity_count();
    egui::Window::new("Debug").show(context, |ui| {
        ui.label(format!("FPS: {:.1}", fps));
        ui.label(format!("Entities: {}", n_entities));
    });
}
