use crate::{
    engine::{ ecs::{ resources::debug::ShowDebugPanel, world::World }, fps_counter::FpsCounter },
    game::input::{ actions::Action, world_ext::InputWorldExt },
};

pub fn debug_panel(context: &egui::Context, world: &mut World) {
    let toggle_pressed = {
        let input = world.input_state();
        let key_bindings = world.key_bindings();
        key_bindings.is_action_just_pressed(&Action::ToggleDebugPanel, &input)
    };

    let show_panel = {
        let panel = world.get_resource_mut::<ShowDebugPanel>().unwrap();
        if toggle_pressed {
            panel.0 = !panel.0;
        }
        panel.0
    };

    if !show_panel {
        return;
    }

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
