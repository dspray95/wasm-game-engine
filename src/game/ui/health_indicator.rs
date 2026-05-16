use egui::Color32;

use crate::{
    engine::{
        ecs::{components::transform::Transform, world::World},
        ui::projection::world_to_screen,
    },
    game::components::player::Player,
};

const MAX_HP: i32 = 3;
const PIP_GLYPH: &str = "^";
const PIP_FONT_SIZE: f32 = 22.0;
const PIP_HORIZONTAL_OFFSET: f32 = 26.0;
const SCREEN_OFFSET_BELOW_PLAYER_PIXELS: f32 = 50.0;

pub fn health_indicator(context: &egui::Context, world: &mut World) {
    let Some((player_entity_id, health)) = world
        .iter_component::<Player>()
        .next()
        .map(|(id, player)| (id, player.health))
    else {
        return;
    };
    let Some(player_position) = world
        .get_component_by_id::<Transform>(player_entity_id)
        .map(|t| t.position)
    else {
        return;
    };

    // Project the player's world position to screen pixels using the active
    // camera, then push the row of pips down by a fixed pixel offset so they
    // sit beneath the ship rather than over it. We offset in screen-space
    // pixels (not world units) because the camera sits close to the ship's y
    // level — any meaningful world-space offset below the player would
    // project off the bottom of the screen at this FOV.
    let Some(player_screen) = world_to_screen(world, player_position, context.screen_rect()) else {
        return;
    };
    let screen_x = player_screen.x;
    let screen_y = player_screen.y + SCREEN_OFFSET_BELOW_PLAYER_PIXELS;

    let font_id = egui::FontId::new(PIP_FONT_SIZE, egui::FontFamily::Name("display".into()));
    let alive = Color32::from_rgb(220, 60, 60);
    let lost = Color32::from_rgba_unmultiplied(80, 20, 20, 220);

    // Anchor the area so the row of pips is horizontally centred on screen_x.
    // The left edge sits half a row's width to the left of centre, with a
    // small fudge for the glyph's own left-bearing.
    egui::Area::new(egui::Id::new("health_indicator"))
        .fixed_pos(egui::pos2(
            screen_x - (MAX_HP as f32 - 1.0) * 0.5 * PIP_HORIZONTAL_OFFSET - 10.0,
            screen_y,
        ))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let cursor = ui.cursor().left_top();
            for i in 0..MAX_HP {
                let dx = (i as f32) * PIP_HORIZONTAL_OFFSET;
                let color = if i < health { alive } else { lost };
                ui.painter().text(
                    egui::pos2(cursor.x + dx, cursor.y),
                    egui::Align2::LEFT_TOP,
                    PIP_GLYPH,
                    font_id.clone(),
                    color,
                );
            }
            ui.allocate_exact_size(
                egui::vec2(
                    MAX_HP as f32 * PIP_HORIZONTAL_OFFSET,
                    PIP_FONT_SIZE * 1.2,
                ),
                egui::Sense::hover(),
            );
        });
}
