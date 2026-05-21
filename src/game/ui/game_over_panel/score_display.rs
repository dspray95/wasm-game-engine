use egui::Color32;

use crate::game::ui::game_over_panel::fonts::PanelFonts;

pub fn draw(ui: &mut egui::Ui, fonts: &PanelFonts, final_score: i32) {
    ui.label(
        egui::RichText::new("YOUR SCORE")
            .font(fonts.title.clone())
            .color(Color32::from_rgb(0, 220, 255)),
    );
    ui.label(
        egui::RichText::new(format!("{:09}", final_score))
            .font(fonts.score.clone())
            .color(Color32::WHITE),
    );
}
