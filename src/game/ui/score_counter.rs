use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            player_score::PlayerScore,
            screen_effects::ScreenEffects,
        },
        ui::theme::{chromatic_aberration, CanyonColors},
    },
};

const MAX_GLITCH_OFFSET_PIXELS: f32 = 6.0;
const FONT_SIZE: f32 = 18.0;

pub fn score_counter(context: &egui::Context, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|s| s.phase)
        .unwrap_or(GameOverPhase::Playing);
    if !matches!(phase, GameOverPhase::Playing | GameOverPhase::DeathRamping) {
        return;
    }

    let score = world
        .get_resource::<PlayerScore>()
        .map(|player_score| player_score.score)
        .unwrap_or(0);

    let glitch_intensity = world
        .get_resource::<ScreenEffects>()
        .map(|effects| effects.glitch_intensity())
        .unwrap_or(0.0);

    let (theme, colors) = context.design::<CanyonColors>();
    let text = format!("{score:09}");

    Styled::area()
        .id("score_counter")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
        .show(context, |ui| {
            let mut label = Styled::label(&text)
                .font(theme.font_display(FONT_SIZE))
                .text_color(colors.text)
                .extend();
            // RGB split widens with the screen-glitch intensity.
            if glitch_intensity > 0.0 {
                let offset = MAX_GLITCH_OFFSET_PIXELS * glitch_intensity;
                label = label.apply(chromatic_aberration(&colors, offset));
            }
            label.show(ui);
        });
}
