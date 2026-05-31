use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            tutorial_state::TutorialState,
        },
        ui::theme::{chromatic_aberration, CanyonColors},
    },
};

const FONT_SIZE: f32 = 16.0;
const ABERRATION_OFFSET: f32 = 2.0;

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

    // Each segment is (text, aberrate?) — the bracketed key tokens get the
    // chromatic-aberration split, the surrounding words stay plain white.
    let segments: Vec<(&str, bool)> = if !move_done {
        vec![("MOVE WITH", false), ("[A]", true), ("+", false), ("[D]", true)]
    } else if !fired {
        vec![("FIRE WITH", false), ("[SPACE]", true)]
    } else {
        return;
    };

    let (theme, colors) = context.design::<CanyonColors>();
    let font = theme.font_display(FONT_SIZE);

    Styled::area()
        .id("tutorial_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            Styled::row()
                .gap(FONT_SIZE * 0.35)
                .align(egui::Align::Center)
                .show(ui, |ui| {
                    for (text, aberrate) in segments {
                        let mut label = Styled::label(text)
                            .font(font.clone())
                            .text_color(egui::Color32::WHITE)
                            .extend();
                        if aberrate {
                            label = label.apply(chromatic_aberration(&colors, ABERRATION_OFFSET));
                        }
                        label.show(ui);
                    }
                });
        });
}
