use egui::Color32;

use crate::game::{
    resources::high_scores::MAX_ENTRIES, ui::game_over_panel::fonts::PanelFonts,
};

pub fn draw(
    ui: &mut egui::Ui,
    fonts: &PanelFonts,
    entries: &[(String, i32)],
    submitted_index: Option<usize>,
) {
    ui.label(
        egui::RichText::new("HIGH SCORES")
            .font(fonts.title.clone())
            .color(Color32::from_rgb(0, 220, 255)),
    );
    ui.add_space(4.0);

    for slot in 0..MAX_ENTRIES {
        let (initials, score) = entries
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ("---".to_string(), 0));
        let color = if submitted_index == Some(slot) {
            Color32::from_rgb(255, 215, 0)
        } else {
            Color32::WHITE
        };
        let text = format!("{:>2}.  {}   {:09}", slot + 1, initials, score);
        ui.label(
            egui::RichText::new(text)
                .font(fonts.row.clone())
                .color(color),
        );
    }
}
