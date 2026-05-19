use egui::Color32;

use crate::{
    engine::ecs::world::World, game::resources::screen_effects::ScreenEffects,
};

/// Full-screen white rectangle whose alpha fades out alongside
/// `ScreenEffects.white_flash_timer`. The bomb detonation triggers it for a
/// short pulse; otherwise the panel paints nothing and costs nothing.
pub fn white_flash_overlay(context: &egui::Context, world: &mut World) {
    let intensity = world
        .get_resource::<ScreenEffects>()
        .map(|s| s.white_flash_intensity())
        .unwrap_or(0.0);
    if intensity <= 0.0 {
        return;
    }

    let alpha = (intensity * 255.0).round().clamp(0.0, 255.0) as u8;
    let screen = context.screen_rect();

    egui::Area::new(egui::Id::new("white_flash_overlay"))
        .fixed_pos(screen.left_top())
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(context, |ui| {
            ui.painter().rect_filled(
                screen,
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
            );
            ui.allocate_exact_size(screen.size(), egui::Sense::hover());
        });
}
