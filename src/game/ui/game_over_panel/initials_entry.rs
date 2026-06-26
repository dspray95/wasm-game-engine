use egui_styled::prelude::*;
use egui_styled::theme::StyledTheme;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            high_scores::{sanitize_initials, HighScores, INITIALS_LEN},
            profanity::ProfanityList,
        },
        systems::high_score_sync_system::request_submit,
        ui::theme::CanyonColors,
    },
};

pub fn draw(
    ui: &mut egui::Ui,
    theme: &StyledTheme,
    scale: f32,
    world: &mut World,
    final_score: i32,
    visible: bool,
) {
    let colors = ui.ctx().design_data::<CanyonColors>();
    let title_font = theme.font_display(theme.font_size_md);
    let row_font = theme.font_display(theme.font_size_sm);

    let mut buffer = world
        .get_resource::<GameOverState>()
        .map(|state| state.initials_buffer.clone())
        .unwrap_or_default();

    Styled::label("ENTER INITIALS")
        .font(title_font)
        .text_color(colors.input_magenta)
        .visible(visible)
        .show(ui);

    // Strip non-letters from the input events *before* the TextEdit consumes
    // them — post-filtering the buffer lets a typed digit paint for one frame
    // first, which reads as a flash. Also uppercases here so casing never
    // flickers either.
    if visible {
        ui.input_mut(|input| {
            input.events.retain_mut(|event| match event {
                egui::Event::Text(text) | egui::Event::Paste(text) => {
                    *text = text
                        .chars()
                        .filter(|character| character.is_ascii_alphabetic())
                        .map(|character| character.to_ascii_uppercase())
                        .collect();
                    !text.is_empty()
                }
                _ => true,
            });
        });
    }

    let response = Styled::text_edit(&mut buffer)
        .char_limit(INITIALS_LEN)
        .font(row_font.clone())
        .desired_width(120.0 * scale)
        .horizontal_align(egui::Align::Center)
        .bg(colors.background)
        .text_color(colors.text)
        .border(0.0, egui::Color32::TRANSPARENT)
        .focus_border(0.0, egui::Color32::TRANSPARENT)
        .corner_radius(theme.rounding_sm)
        .padding(egui::Margin {
            left: 2,
            right: 2,
            top: (10.0 * scale).round() as i8,
            bottom: (7.0 * scale).round() as i8,
        })
        .visible(visible)
        .show(ui);

    let submit_clicked = Styled::button("SUBMIT")
        .font(row_font.clone())
        .bg(egui::Color32::TRANSPARENT)
        .hover_bg(colors.panel_elevated)
        .text_color(colors.hud_cyan)
        .border(4.0, colors.hud_cyan)
        .hover_border(4.0, colors.hud_cyan_bright)
        .corner_radius(0)
        .min_width(180.0 * scale)
        .min_height(36.0 * scale)
        .margin_top(theme.spacing_md)
        .shadow(egui::vec2(3.0, -2.0), 4.0, colors.input_magenta)
        .shadow(egui::vec2(-2.0, 3.0), 4.0, egui::Color32::WHITE)
        .visible(visible)
        .show(ui)
        .clicked();

    let cleaned: String = buffer
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .map(|character| character.to_ascii_uppercase())
        .take(INITIALS_LEN)
        .collect();

    let previous_buffer = world
        .get_resource::<GameOverState>()
        .map(|state| state.initials_buffer.clone())
        .unwrap_or_default();

    let buffer_changed = cleaned != previous_buffer;
    if let Some(state) = world.get_resource_mut::<GameOverState>() {
        state.initials_buffer = cleaned.clone();
        if buffer_changed {
            state.entry_error = None;
        }
    }
    // Submit on Enter via egui's own signal: a single-line TextEdit surrenders
    // focus when Enter is pressed, which surfaces as `lost_focus()`. This is the
    // cross-platform-correct detection (raw key checks miss it on web, where the
    // focused field routes input through the browser text agent). The earlier
    // code's `has_focus()` gate was the bug — focus is already gone on this frame.
    let enter_pressed = response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

    // Keep the field focused for typing, but only re-grab when it isn't already
    // focused — re-grabbing every frame suppresses the `lost_focus()` above.
    if visible && !response.has_focus() && !enter_pressed {
        response.request_focus();
    }

    if (enter_pressed || submit_clicked) && !cleaned.is_empty() {
        // Strip the Enter so the Showing-phase play-again prompt can't see it on
        // the frames right after we flip the phase.
        ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        submit(world, &cleaned, final_score);
    }

    let entry_error = world
        .get_resource::<GameOverState>()
        .and_then(|state| state.entry_error);
    if let Some(message) = entry_error {
        Styled::label(message)
            .font(row_font)
            .text_color(colors.danger_red)
            .margin_top(theme.spacing_sm)
            .show(ui);
    }
}

fn submit(world: &mut World, raw_initials: &str, final_score: i32) {
    let initials = sanitize_initials(raw_initials);
    let is_blocked = world
        .get_resource::<ProfanityList>()
        .map(|profanity| profanity.is_blocked(&initials))
        .unwrap_or(false);
    if is_blocked {
        if let Some(state) = world.get_resource_mut::<GameOverState>() {
            state.entry_error = Some("NOT ALLOWED");
        }
        return;
    }
    let placed_index = world
        .get_resource_mut::<HighScores>()
        .and_then(|high_scores| {
            let index = high_scores.insert(&initials, final_score);
            high_scores.save();
            index
        });
    request_submit(world, initials, final_score);
    if let Some(state) = world.get_resource_mut::<GameOverState>() {
        state.phase = GameOverPhase::Showing;
        state.submitted_index = placed_index;
        state.initials_buffer.clear();
    }
}
