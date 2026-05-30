use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            tutorial_state::TutorialState,
        },
        ui::theme::CanyonColors,
    },
};

const FONT_SIZE: f32 = 16.0;

pub fn tutorial_panel(context: &egui::Context, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|s| s.phase)
        .unwrap_or(GameOverPhase::Playing);
    if !matches!(phase, GameOverPhase::Playing) {
        return;
    }

    let tutorial = match world.get_resource::<TutorialState>() {
        Some(t) => (t.move_done(), t.fired, t.completed),
        None => return,
    };
    let (move_done, fired, completed) = tutorial;

    if completed {
        return;
    }

    let prompt = if !move_done {
        "MOVE WITH [A] + [D]"
    } else if !fired {
        "FIRE WITH [SPACE]"
    } else {
        return;
    };

    let (theme, _colors) = context.design::<CanyonColors>();

    Styled::area()
        .id("tutorial_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            Styled::label(prompt)
                .font(theme.font_display(FONT_SIZE))
                .text_color(egui::Color32::WHITE)
                .extend()
                .show(ui);
        });
}
