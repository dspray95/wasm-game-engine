use egui::Color32;

use crate::engine::ecs::{
    resources::toasts::{ToastAnchor, ToastQueue},
    world::World,
};

const LINE_HEIGHT: f32 = 22.0;
const ANCHOR_Y_OFFSET: f32 = 90.0;
const FONT_SIZE: f32 = 16.0;

// Slide animation knobs.
//
// A new toast is *seeded* SLIDE_IN_OFFSET pixels above its target slot on
// its first render; on subsequent frames `animate_value_with_time` smooths
// it toward the actual slot position. The same machinery handles existing
// toasts whose slots have shifted because a newer one displaced them
// (e.g. slot 0 → slot 1) — they slide together with the new arrival.
const SLIDE_IN_OFFSET: f32 = 18.0;
const SLIDE_DURATION_SECONDS: f32 = 0.12;

pub fn toast_panel(context: &egui::Context, world: &mut World) {
    // Pull a snapshot (id, text, alpha) of every visible HUD toast, in slot
    // order (newest first). We drop the world borrow before talking to egui
    // so the animation calls can freely poke at `ctx.memory_mut`.
    let snapshots: Vec<ToastSnapshot> = {
        let Some(queue) = world.get_resource::<ToastQueue>() else {
            return;
        };
        let mut visible: Vec<&crate::engine::ecs::resources::toasts::Toast> = queue
            .toasts
            .iter()
            .filter(|t| matches!(t.anchor, ToastAnchor::HudStack) && !t.is_dormant())
            .collect();
        visible.sort_by(|a, b| {
            a.elapsed
                .partial_cmp(&b.elapsed)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        visible
            .iter()
            .map(|t| ToastSnapshot {
                id: t.id,
                text: t.text.clone(),
                alpha_byte: (t.alpha() * 255.0).round() as u8,
            })
            .collect()
    };

    let font_id = egui::FontId::new(FONT_SIZE, egui::FontFamily::Name("display".into()));

    egui::Area::new(egui::Id::new("toast_panel_hud"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, ANCHOR_Y_OFFSET))
        .show(context, |ui| {
            let cursor = ui.cursor().left_top();
            for (slot, snapshot) in snapshots.iter().enumerate() {
                if snapshot.alpha_byte == 0 {
                    continue;
                }
                let target_y = (slot as f32) * LINE_HEIGHT;
                let y = animated_slide_y(context, snapshot.id, target_y);

                let colour =
                    Color32::from_rgba_unmultiplied(255, 255, 255, snapshot.alpha_byte);
                ui.painter().text(
                    egui::pos2(cursor.x, cursor.y + y),
                    egui::Align2::CENTER_TOP,
                    &snapshot.text,
                    font_id.clone(),
                    colour,
                );
            }
            let reserved = ((snapshots.len() as f32 + 1.0) * LINE_HEIGHT).max(LINE_HEIGHT);
            ui.allocate_exact_size(egui::vec2(1.0, reserved), egui::Sense::hover());
        });
}

struct ToastSnapshot {
    id: u64,
    text: String,
    alpha_byte: u8,
}

/// Smoothly tween toward `target_y` via egui's animation machinery. On the
/// toast's first render we set `placed` in egui memory and *seed* the
/// animator with a starting value `SLIDE_IN_OFFSET` pixels above the
/// target; on subsequent renders we just feed in the real target and egui
/// interpolates from wherever it was. The animation state cleans itself up
/// naturally because the animator entry is keyed by the toast's stable id
/// and falls out of egui's memory once nothing references it again.
fn animated_slide_y(context: &egui::Context, toast_id: u64, target_y: f32) -> f32 {
    let animate_id = egui::Id::new(("toast_y", toast_id));
    let placed_id = egui::Id::new(("toast_placed", toast_id));

    let already_placed = context.memory(|m| m.data.get_temp::<bool>(placed_id).unwrap_or(false));

    let value_for_this_frame = if already_placed {
        target_y
    } else {
        // First render: seed the animator above the target slot so the
        // next frame's lerp produces the slide-in motion.
        target_y - SLIDE_IN_OFFSET
    };

    let y = context.animate_value_with_time(animate_id, value_for_this_frame, SLIDE_DURATION_SECONDS);

    if !already_placed {
        context.memory_mut(|m| m.data.insert_temp(placed_id, true));
    }
    y
}
