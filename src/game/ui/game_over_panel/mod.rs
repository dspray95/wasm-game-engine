mod backdrop;
mod initials_entry;
mod leaderboard;
mod play_again;
mod score_display;

use egui_styled::prelude::*;
use egui_styled::theme::StyledTheme;

use crate::{
    engine::ecs::world::World,
    game::resources::{
        game_over_state::{GameOverPhase, GameOverState},
        high_scores::HighScores,
    },
};

// Viewport height at which the panel is laid out at its native size. Below
// this we scale fonts and spacing uniformly so the leaderboard never overflows.
const REFERENCE_VIEWPORT_HEIGHT: f32 = 900.0;
const MIN_PANEL_SCALE: f32 = 0.55;

pub fn game_over_panel(context: &egui::Context, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|state| state.phase)
        .unwrap_or(GameOverPhase::Playing);

    if matches!(phase, GameOverPhase::PreStart | GameOverPhase::Playing | GameOverPhase::DeathRamping) {
        return;
    }

    let final_score = world
        .get_resource::<GameOverState>()
        .map(|state| state.final_score)
        .unwrap_or(0);

    let now = context.input(|input| input.time);
    let reveal_start = world
        .get_resource_mut::<GameOverState>()
        .map(|state| *state.reveal_start_time.get_or_insert(now))
        .unwrap_or(now);
    let reveal_elapsed = (now - reveal_start) as f32;
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

    let base_theme = context.styled_theme();
    let screen_height = context.content_rect().height();
    let panel_scale = (screen_height / REFERENCE_VIEWPORT_HEIGHT).clamp(MIN_PANEL_SCALE, 1.0);
    let theme = scaled_theme(&base_theme, panel_scale);

    Styled::area()
        .id("game_over_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(context, |ui| {
            Styled::column()
                .gap(theme.spacing_lg)
                .align(egui::Align::Center)
                .show(ui, |ui| {
                    score_display::draw(ui, &theme, final_score, reveal_elapsed);
                    initials_entry::draw(
                        ui,
                        &theme,
                        panel_scale,
                        world,
                        final_score,
                        phase == GameOverPhase::EnteringInitials,
                    );
                    leaderboard::draw(ui, &theme, &entries, submitted_index);
                    play_again::draw(ui, &theme, panel_scale, world);
                });
        });
}

fn scaled_theme(base: &StyledTheme, scale: f32) -> StyledTheme {
    let mut theme = base.clone();
    theme.spacing_xs *= scale;
    theme.spacing_sm *= scale;
    theme.spacing_md *= scale;
    theme.spacing_lg *= scale;
    theme.spacing_xl *= scale;
    theme.font_size_sm *= scale;
    theme.font_size_md *= scale;
    theme.font_size_lg *= scale;
    theme.font_size_xl *= scale;
    theme
}