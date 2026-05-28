use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::game_over_state::{GameOverPhase, GameOverState},
        ui::theme::CanyonColors,
    },
};

const BLINK_PERIOD_SECONDS: f64 = 1.0;
const STATIC_GLITCH_OFFSET_PIXELS: f32 = 6.0;

pub fn draw(ui: &mut egui::Ui, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|state| state.phase)
        .unwrap_or(GameOverPhase::Playing);

    let (theme, colors) = ui.ctx().design::<CanyonColors>();

    let row_font = theme.font_display(theme.font_size_sm);
    let row_height = row_font.size + 4.0;

    let now = ui.input(|input| input.time);
    // Hidden during initials entry — Enter is bound to submit there, and the
    // prompt would compete visually with the SUBMIT button.
    let showing = phase == GameOverPhase::Showing;
    let blink = (now % BLINK_PERIOD_SECONDS) < (BLINK_PERIOD_SECONDS / 2.0);

    Styled::row()
        .align(egui::Align::Center)
        .gap(0.0)
        .min_height(row_height * 2.0)
        .margin_top(20.0)
        .visible(showing && blink)
        .show(ui, |ui| {
            Styled::label("PRESS ")
                .font(row_font.clone())
                .text_color(colors.text)
                .wrap(false)
                .show(ui);

            let enter_text = "[ENTER]";
            let galley = ui.painter().layout_no_wrap(
                enter_text.to_owned(),
                row_font.clone(),
                colors.text,
            );
            let (rect, _response) =
                ui.allocate_exact_size(galley.size(), egui::Sense::hover());
            let origin = rect.left_top();
            ui.painter().text(
                origin + egui::vec2(-STATIC_GLITCH_OFFSET_PIXELS, 0.0),
                egui::Align2::LEFT_TOP,
                enter_text,
                row_font.clone(),
                colors.hud_cyan,
            );
            ui.painter().text(
                origin + egui::vec2(STATIC_GLITCH_OFFSET_PIXELS, 0.0),
                egui::Align2::LEFT_TOP,
                enter_text,
                row_font.clone(),
                colors.input_magenta,
            );
            ui.painter().text(
                origin,
                egui::Align2::LEFT_TOP,
                enter_text,
                row_font.clone(),
                colors.text,
            );

            Styled::label(" TO PLAY AGAIN")
                .font(row_font)
                .text_color(colors.text)
                .wrap(false)
                .show(ui);
        });

    if !showing {
        return;
    }

    let enter_down = ui.input(|input| input.key_down(egui::Key::Enter));
    let enter_pressed = ui.input(|input| input.key_pressed(egui::Key::Enter));
    if let Some(state) = world.get_resource_mut::<GameOverState>() {
        // Arm only after Enter is released, so a held/repeated press carried
        // over from the initials submit can't restart on the next frame.
        if !enter_down {
            state.restart_armed = true;
        }
        if state.restart_armed && enter_pressed {
            state.restart_requested = true;
        }
    }
}
