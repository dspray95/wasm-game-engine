mod backdrop;
mod initials_entry;
mod leaderboard;
mod play_again;
mod score_display;

use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::resources::{
        game_over_state::{GameOverPhase, GameOverState},
        high_scores::HighScores,
    },
};

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

    let theme = context.styled_theme();

    Styled::area()
        .id("game_over_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(context, |ui| {
            Styled::column()
                .gap(theme.spacing_lg)
                .align(egui::Align::Center)
                .show(ui, |ui| {
                    score_display::draw(ui, final_score);
                    initials_entry::draw(ui, world, final_score, phase == GameOverPhase::EnteringInitials);
                    leaderboard::draw(ui, &entries, submitted_index);
                    play_again::draw(ui, world);
                });
        });
}
