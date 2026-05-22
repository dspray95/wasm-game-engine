use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::game_over_state::{GameOverPhase, GameOverState},
        ui::theme::CanyonColors,
    },
};

const BLINK_PERIOD_SECONDS: f64 = 1.0;

pub fn draw(ui: &mut egui::Ui, world: &mut World) {
    // Skip during initials entry — Enter is bound to submit there, and the
    // prompt would compete visually with the SUBMIT button.
    let phase = world
        .get_resource::<GameOverState>()
        .map(|state| state.phase);
    if phase != Some(GameOverPhase::Showing) {
        return;
    }

    let (theme, colors) = ui.ctx().design::<CanyonColors>();
    let row_font = theme.font_display(theme.font_size_sm);

    // Square-wave blink: visible for the first half of each period, hidden
    // for the second. egui's input time is monotonic from app start.
    let now = ui.input(|input| input.time);
    let visible = (now % BLINK_PERIOD_SECONDS) < (BLINK_PERIOD_SECONDS / 2.0);

    // Reserve a fixed-size slot once so the panel doesn't reflow on each
    // blink frame. We then paint the text into that rect ourselves only when
    // visible — `ui.label` would re-measure per frame and bump everything
    // above it.
    let text = "PRESS [ENTER] TO PLAY AGAIN";
    let row_height = row_font.size + 4.0;
    let (rect, _response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::hover());
    if visible {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            row_font,
            colors.text,
        );
    }

    if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
        if let Some(state) = world.get_resource_mut::<GameOverState>() {
            state.restart_requested = true;
        }
    }

    // Keep the panel repainting so the blink stays animated even when no
    // other input is happening.
    ui.ctx().request_repaint();
}
