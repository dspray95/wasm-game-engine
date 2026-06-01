use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::game_over_state::{GameOverPhase, GameOverState},
        ui::theme::{chromatic_aberration, CanyonColors},
    },
};

const BLINK_PERIOD_SECONDS: f64 = 1.2;
const FONT_SIZE: f32 = 16.0;
// Horizontal RGB split, in pixels, for the chromatic aberration on [ENTER].
const ABERRATION_OFFSET: f32 = 2.0;

pub fn pre_start_panel(context: &egui::Context, world: &mut World) {
    let phase = world
        .get_resource::<GameOverState>()
        .map(|s| s.phase)
        .unwrap_or(GameOverPhase::Playing);
    if !matches!(phase, GameOverPhase::PreStart) {
        return;
    }

    let enter_pressed = context.input(|i| i.key_pressed(egui::Key::Enter));
    if enter_pressed {
        if let Some(state) = world.get_resource_mut::<GameOverState>() {
            state.phase = GameOverPhase::Playing;
        }
        // Let the host page react to the game starting (e.g. hide intro DOM).
        notify_game_start();
        return;
    }

    let now = context.input(|i| i.time);
    if (now % BLINK_PERIOD_SECONDS) >= (BLINK_PERIOD_SECONDS * 0.7) {
        context.request_repaint();
        return;
    }
    context.request_repaint();

    let (theme, colors) = context.design::<CanyonColors>();
    let font = theme.font_display(FONT_SIZE);

    Styled::area()
        .id("pre_start_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            // One word-space of gap between segments, independent of how the
            // galley measures trailing whitespace.
            Styled::row()
                .gap(FONT_SIZE * 0.35)
                .align(egui::Align::Center)
                .show(ui, |ui| {
                    Styled::label("PRESS")
                        .font(font.clone())
                        .text_color(egui::Color32::WHITE)
                        .extend()
                        .show(ui);
                    Styled::label("[ENTER]")
                        .font(font.clone())
                        .text_color(egui::Color32::WHITE)
                        .apply(chromatic_aberration(&colors, ABERRATION_OFFSET))
                        .extend()
                        .show(ui);
                    Styled::label("TO START")
                        .font(font.clone())
                        .text_color(egui::Color32::WHITE)
                        .extend()
                        .show(ui);
                });
        });
}

/// Notify the host page that the game has started, so it can hide intro DOM
/// elements. Calls `window.onGameStart()` if the page defines it; `catch` makes
/// a missing function a no-op rather than a panic. No-op on native.
#[cfg(target_arch = "wasm32")]
fn notify_game_start() {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window, catch)]
        fn onGameStart() -> Result<(), JsValue>;
    }
    let _ = onGameStart();
}

#[cfg(not(target_arch = "wasm32"))]
fn notify_game_start() {}
