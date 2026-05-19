use egui::Color32;

use crate::engine::ecs::{
    resources::toasts::{ToastAnchor, ToastQueue},
    world::World,
};

const LINE_HEIGHT: f32 = 22.0;
const ANCHOR_Y_OFFSET: f32 = 90.0;
const FONT_SIZE: f32 = 16.0;

pub fn toast_panel(context: &egui::Context, world: &mut World) {
    let Some(queue) = world.get_resource::<ToastQueue>() else {
        return;
    };

    let font_id = egui::FontId::new(FONT_SIZE, egui::FontFamily::Name("display".into()));

    // HUD-stack toasts: newest at the top (slot 0), older ones below.
    // Sort by elapsed ascending — earliest-elapsed first.
    let mut hud: Vec<&crate::engine::ecs::resources::toasts::Toast> = queue
        .toasts
        .iter()
        .filter(|t| matches!(t.anchor, ToastAnchor::HudStack))
        .collect();
    hud.sort_by(|a, b| {
        a.elapsed
            .partial_cmp(&b.elapsed)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    egui::Area::new(egui::Id::new("toast_panel_hud"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, ANCHOR_Y_OFFSET))
        .show(context, |ui| {
            let cursor = ui.cursor().left_top();
            for (slot, toast) in hud.iter().enumerate() {
                let alpha = (toast.alpha() * 255.0).round() as u8;
                if alpha == 0 {
                    continue;
                }
                let colour = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                ui.painter().text(
                    egui::pos2(cursor.x, cursor.y + (slot as f32) * LINE_HEIGHT),
                    egui::Align2::CENTER_TOP,
                    &toast.text,
                    font_id.clone(),
                    colour,
                );
            }
            // Allocate enough space so egui's auto-shrink doesn't clip our painter draws.
            ui.allocate_exact_size(
                egui::vec2(1.0, (hud.len() as f32 * LINE_HEIGHT).max(LINE_HEIGHT)),
                egui::Sense::hover(),
            );
        });

    // World-anchored toasts: data model is ready; rendering will land when
    // the first diegetic use case ("+100 above killed enemies") is built.
    // Until then any World-anchored toast in the queue ticks/expires
    // normally but renders nothing.
    // TODO: spawn float-up animation when needed.
}
