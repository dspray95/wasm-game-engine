use egui::Color32;

use crate::{
    engine::ecs::world::World,
    game::{
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            high_scores::{sanitize_initials, HighScores, INITIALS_LEN},
            profanity::ProfanityList,
        },
        ui::game_over_panel::fonts::PanelFonts,
    },
};

pub fn draw(ui: &mut egui::Ui, fonts: &PanelFonts, world: &mut World, final_score: i32) {
    ui.label(
        egui::RichText::new("ENTER INITIALS")
            .font(fonts.title.clone())
            .color(Color32::from_rgb(255, 0, 200)),
    );

    let mut buffer = world
        .get_resource::<GameOverState>()
        .map(|state| state.initials_buffer.clone())
        .unwrap_or_default();

    let response = ui.add(
        egui::TextEdit::singleline(&mut buffer)
            .char_limit(INITIALS_LEN)
            .font(fonts.row.clone())
            .desired_width(120.0)
            .horizontal_align(egui::Align::Center),
    );

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

    ui.add_space(8.0);
    let submit_clicked = ui
        .add_sized(
            [180.0, 36.0],
            egui::Button::new(egui::RichText::new("SUBMIT").font(fonts.row.clone())),
        )
        .clicked();

    if (enter_pressed || submit_clicked) && !cleaned.is_empty() {
        submit(world, &cleaned, final_score);
    }

    let entry_error = world
        .get_resource::<GameOverState>()
        .and_then(|state| state.entry_error);
    if let Some(message) = entry_error {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(message)
                .font(fonts.row.clone())
                .color(Color32::from_rgb(255, 80, 80)),
        );
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
