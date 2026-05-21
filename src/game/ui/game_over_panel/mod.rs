mod backdrop;
mod fonts;
mod initials_entry;
mod leaderboard;
mod play_again;
mod score_display;

use crate::{
    engine::ecs::world::World,
    game::resources::{
        game_over_state::{GameOverPhase, GameOverState},
        high_scores::HighScores,
    },
};

use fonts::PanelFonts;

pub fn game_over_panel(context: &egui::Context, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|state| state.phase)
        .unwrap_or(GameOverPhase::Playing);

    if matches!(phase, GameOverPhase::Playing | GameOverPhase::DeathRamping) {
        return;
    }

    let final_score = world
        .get_resource::<GameOverState>()
        .map(|state| state.final_score)
        .unwrap_or(0);
    let submitted_index = world
        .get_resource::<GameOverState>()
        .and_then(|state| state.submitted_index);
    let entries: Vec<(String, i32)> = world
        .get_resource::<HighScores>()
        .map(|high_scores| {
            high_scores
                .entries
                .iter()
                .map(|entry| (entry.initials.clone(), entry.score))
                .collect()
        })
        .unwrap_or_default();

    backdrop::draw(context);

    let fonts = PanelFonts::new();

    egui::Area::new(egui::Id::new("game_over_panel"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                score_display::draw(ui, &fonts, final_score);
                ui.add_space(16.0);

                if phase == GameOverPhase::EnteringInitials {
                    initials_entry::draw(ui, &fonts, world, final_score);
                    ui.add_space(16.0);
                }

                leaderboard::draw(ui, &fonts, &entries, submitted_index);
                ui.add_space(20.0);
                play_again::draw(ui, &fonts, world);
            });
        });
}
