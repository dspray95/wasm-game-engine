use cgmath::{InnerSpace, Vector3};
use egui::Color32;
use egui_styled::prelude::*;

use crate::{
    engine::{
        ecs::{
            components::world_transform::WorldTransform,
            resources::toasts::{ToastAnchor, ToastQueue},
            world::World,
        },
        ui::projection::world_to_screen,
    },
    game::{components::player::Player, ui::theme::CanyonColors},
};

const FONT_SIZE: f32 = 12.0;
// Extra upward drift in screen pixels over the toast's life, on top of its
// world anchor — gives the pop a little rise as it fades.
const RISE_PIXELS_PER_SECOND: f32 = 28.0;
// Distance-based shrink: kills within this distance of the player render at
// full size; beyond it the pop scales as ~1/distance (perspective-like),
// clamped so far kills stay legible.
const FULL_SIZE_DISTANCE: f32 = 10.0;
const MIN_SCALE: f32 = 0.45;

struct PopupSnapshot {
    id: u64,
    text: String,
    world_position: Vector3<f32>,
    alpha_byte: u8,
    elapsed: f32,
}

pub fn score_popup_panel(context: &egui::Context, world: &mut World) {
    let popups: Vec<PopupSnapshot> = {
        let Some(queue) = world.get_resource::<ToastQueue>() else {
            return;
        };
        queue
            .toasts
            .iter()
            .filter(|toast| !toast.is_dormant())
            .filter_map(|toast| match toast.anchor {
                ToastAnchor::World(position) => Some(PopupSnapshot {
                    id: toast.id,
                    text: toast.text.clone(),
                    world_position: position,
                    alpha_byte: (toast.alpha() * 255.0).round() as u8,
                    elapsed: toast.elapsed,
                }),
                ToastAnchor::HudStack => None,
            })
            .collect()
    };
    if popups.is_empty() {
        return;
    }

    let (theme, _colors) = context.design::<CanyonColors>();
    let screen_rect = context.content_rect();

    let player_position = world
        .iter_component::<Player>()
        .next()
        .map(|(id, _)| id)
        .and_then(|id| world.get_component_by_id::<WorldTransform>(id))
        .map(|transform| transform.position);

    for popup in popups {
        if popup.alpha_byte == 0 {
            continue;
        }
        let Some(mut position) = world_to_screen(world, popup.world_position, screen_rect) else {
            continue;
        };
        position.y -= popup.elapsed * RISE_PIXELS_PER_SECOND;

        let scale = match player_position {
            Some(player) => {
                let distance = (popup.world_position - player).magnitude();
                (FULL_SIZE_DISTANCE / distance.max(FULL_SIZE_DISTANCE)).clamp(MIN_SCALE, 1.0)
            }
            None => 1.0,
        };

        let color = Color32::from_rgba_unmultiplied(
            Color32::WHITE.r(),
            Color32::WHITE.g(),
            Color32::WHITE.b(),
            popup.alpha_byte,
        );

        Styled::area()
            .id(("score_popup", popup.id))
            .fixed_pos_centered(position)
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                Styled::label(&popup.text)
                    .font(theme.font_display(FONT_SIZE * scale))
                    .text_color(color)
                    .extend()
                    .show(ui);
            });
    }
}
