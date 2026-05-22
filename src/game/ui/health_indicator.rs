use egui_styled::prelude::*;

use crate::{
    engine::{
        ecs::{components::world_transform::WorldTransform, world::World},
        ui::projection::world_to_screen,
    },
    game::{
        components::player::Player,
        ui::theme::{CanyonColors, DISPLAY_FONT_VISUAL_X_CORRECTION},
    },
};

const MAX_HP: i32 = 3;
const PIP_GLYPH: &str = "^";
const PIP_FONT_SIZE: f32 = 22.0;
const PIP_GAP: f32 = 8.0;
const SCREEN_OFFSET_BELOW_PLAYER_PIXELS: f32 = 70.0;

pub fn health_indicator(context: &egui::Context, world: &mut World) {
    let Some((player_entity_id, health)) = world
        .iter_component::<Player>()
        .next()
        .map(|(id, player)| (id, player.health))
    else {
        return;
    };
    let Some(player_position) = world
        .get_component_by_id::<WorldTransform>(player_entity_id)
        .map(|t| t.position)
    else {
        return;
    };

    // Project player world position to screen pixels. We offset in
    // screen-space (not world units) because the camera sits close to the
    // ship's y-level — a meaningful world-y offset would project off-screen.
    let Some(player_screen) = world_to_screen(world, player_position, context.content_rect())
    else {
        return;
    };

    let (theme, colors) = context.design::<CanyonColors>();
    let pip_font = theme.font_display(PIP_FONT_SIZE);

    // Lost pips are derived from the alive color rather than being a
    // separate hand-picked token. Tweak `health_alive` and both states
    // stay visually related.
    let alive = colors.health_alive;
    let lost = alive.darken(0.6).with_alpha(220);

    let target = egui::pos2(
        player_screen.x + DISPLAY_FONT_VISUAL_X_CORRECTION,
        player_screen.y + SCREEN_OFFSET_BELOW_PLAYER_PIXELS,
    );

    // `fixed_pos_centered` measures the row on frame N and places it
    // centered on the target on frame N+1. Explicit `id` keeps the cached
    // size attached across frames (auto-ids can shift).
    Styled::area()
        .id("health_indicator")
        .fixed_pos_centered(target)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            Styled::row().gap(PIP_GAP).show(ui, |ui| {
                for i in 0..MAX_HP {
                    let color = if i < health { alive } else { lost };
                    Styled::label(PIP_GLYPH)
                        .font(pip_font.clone())
                        .text_color(color)
                        .show(ui);
                }
            });
        });
}
