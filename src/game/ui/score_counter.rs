use egui::Color32;

use crate::{
    engine::ecs::world::World,
    game::resources::{player_score::PlayerScore, screen_effects::ScreenEffects},
};

const MAX_GLITCH_OFFSET_PIXELS: f32 = 6.0;

pub fn score_counter(context: &egui::Context, world: &mut World) {
    let score = world
        .get_resource::<PlayerScore>()
        .map(|player_score| player_score.score)
        .unwrap_or(0);

    let glitch_intensity = world
        .get_resource::<ScreenEffects>()
        .map(|effects| effects.glitch_intensity())
        .unwrap_or(0.0);

    egui::Area::new(egui::Id::new("score_counter"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
        .show(context, |ui| {
            let text = format!("{:09}", score);
            let font_id =
                egui::FontId::new(18.0, egui::FontFamily::Name("display".into()));
            let cursor = ui.cursor().left_top();

            if glitch_intensity > 0.0 {
                let horizontal_offset = MAX_GLITCH_OFFSET_PIXELS * glitch_intensity;
                ui.painter().text(
                    cursor + egui::vec2(-horizontal_offset, 0.0),
                    egui::Align2::LEFT_TOP,
                    &text,
                    font_id.clone(),
                    Color32::from_rgb(0, 220, 255),
                );
                ui.painter().text(
                    cursor + egui::vec2(horizontal_offset, 0.0),
                    egui::Align2::LEFT_TOP,
                    &text,
                    font_id.clone(),
                    Color32::from_rgb(255, 0, 200),
                );
            }

            let main_rect = ui.painter().text(
                cursor,
                egui::Align2::LEFT_TOP,
                &text,
                font_id,
                Color32::WHITE,
            );
            ui.allocate_rect(main_rect, egui::Sense::hover());
        });
}
