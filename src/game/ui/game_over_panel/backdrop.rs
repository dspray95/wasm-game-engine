use egui_styled::prelude::*;

use crate::game::ui::theme::CanyonColors;

const BACKDROP_ALPHA: u8 = 180;

pub fn draw(context: &egui::Context) {
    let colors = context.design_data::<CanyonColors>();
    let [red, green, blue, _] = colors.background.to_array();
    let fill = egui::Color32::from_rgba_unmultiplied(red, green, blue, BACKDROP_ALPHA);

    let screen_rect = context.content_rect();
    egui::Area::new(egui::Id::new("game_over_backdrop"))
        .order(egui::Order::Background)
        .fixed_pos(screen_rect.min)
        .show(context, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, fill);
        });
}
