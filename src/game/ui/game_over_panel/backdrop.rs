use egui::Color32;

pub fn draw(context: &egui::Context) {
    let screen_rect = context.content_rect();
    egui::Area::new(egui::Id::new("game_over_backdrop"))
        .order(egui::Order::Background)
        .fixed_pos(screen_rect.min)
        .show(context, |ui| {
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });
}
