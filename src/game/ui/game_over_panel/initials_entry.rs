use egui_styled::prelude::*;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            high_scores::{sanitize_initials, HighScores, INITIALS_LEN},
            profanity::ProfanityList,
        },
        ui::theme::CanyonColors,
    },
};

pub fn draw(ui: &mut egui::Ui, world: &mut World, final_score: i32) {
    let theme = ui.ctx().styled_theme();
    let colors = ui.ctx().design_data::<CanyonColors>();
    let title_font = theme.font_display(theme.font_size_md);
    let row_font = theme.font_display(theme.font_size_sm);

    Styled::label("ENTER INITIALS")
        .font(title_font)
        .text_color(colors.input_magenta)
        .show(ui);

    let mut buffer = world
        .get_resource::<GameOverState>()
        .map(|state| state.initials_buffer.clone())
        .unwrap_or_default();

    let response = Styled::text_edit(&mut buffer)
        .char_limit(INITIALS_LEN)
        .font(row_font.clone())
        .desired_width(120.0)
        .horizontal_align(egui::Align::Center)
        .bg(colors.panel_surface)
        .text_color(colors.text)
        .border(1.0, colors.input_border)
        .focus_border(1.0, colors.input_magenta)
        .corner_radius(theme.rounding_sm)
        .show(ui);

    let cleaned: String = buffer
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
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
    response.request_focus();

    // `lost_focus()` never fires while we re-grab focus every frame, so detect
    // Enter against the live focus state instead. `consume_key` strips the
    // event from the queue so the play-again prompt (which listens for Enter
    // in the Showing phase) can't fire on the same frame the player submits.
    let enter_pressed = response.has_focus()
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));

    ui.add_space(theme.spacing_md);
    let submit_clicked = Styled::button(egui::RichText::new("SUBMIT").font(row_font.clone()))
        .bg(egui::Color32::TRANSPARENT)
        .hover_bg(colors.panel_elevated)
        .text_color(colors.hud_cyan)
        .border(1.0, colors.hud_cyan)
        .hover_border(1.0, colors.hud_cyan_bright)
        .corner_radius(theme.rounding_sm)
        .padding(egui::Margin::symmetric(20, 8))
        .min_width(180.0)
        .show(ui)
        .clicked();

    if (enter_pressed || submit_clicked) && !cleaned.is_empty() {
        submit(world, &cleaned, final_score);
    }

    let entry_error = world
        .get_resource::<GameOverState>()
        .and_then(|state| state.entry_error);
    if let Some(message) = entry_error {
        ui.add_space(theme.spacing_sm);
        Styled::label(message)
            .font(row_font)
            .text_color(colors.danger_red)
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
    if let Some(state) = world.get_resource_mut::<GameOverState>() {
        state.phase = GameOverPhase::Showing;
        state.submitted_index = placed_index;
        state.initials_buffer.clear();
    }
}
