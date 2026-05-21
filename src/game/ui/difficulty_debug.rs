use egui::Color32;

use crate::{
    engine::ecs::{resources::debug::ShowDebugPanel, world::World},
    game::resources::{
        enemy_resources::EnemySpawnManager, laser_resources::LaserManager,
        player_score::PlayerScore, player_speed_scaling::PlayerSpeedScaling,
    },
};

pub fn difficulty_debug_panel(context: &egui::Context, world: &mut World) {
    let show_panel = world
        .get_resource::<ShowDebugPanel>()
        .map(|p| p.0)
        .unwrap_or(false);
    if !show_panel {
        return;
    }

    let score = world.get_resource::<PlayerScore>().map(|s| s.score).unwrap_or(0);

    let player_speed = world
        .get_resource::<PlayerSpeedScaling>()
        .map(|s| (s.z_speed.value(score), s.z_speed.base, s.z_speed.cap));
    let spawn_interval = world
        .get_resource::<EnemySpawnManager>()
        .map(|m| (m.spawn_interval.value(score), m.spawn_interval.base, m.spawn_interval.cap));
    let laser_cooldown = world
        .get_resource::<LaserManager>()
        .map(|m| (m.fire_cooldown.value(score), m.fire_cooldown.base, m.fire_cooldown.cap));

    egui::Window::new("Difficulty")
        .fixed_pos([10.0, 90.0])
        .collapsible(false)
        .resizable(false)
        .movable(false)
        .title_bar(false)
        .frame(egui::Frame::window(&context.global_style()).shadow(egui::Shadow::NONE))
        .show(context, |ui| {
            ui.label(egui::RichText::new(format!("Score: {}", score)).color(Color32::WHITE));
            if let Some((current, base, cap)) = player_speed {
                ui.label(
                    egui::RichText::new(format!(
                        "Player speed: {:.2}  ({:.0} \u{2192} {:.0})",
                        current, base, cap
                    ))
                    .color(Color32::WHITE),
                );
            }
            if let Some((current, base, cap)) = spawn_interval {
                ui.label(
                    egui::RichText::new(format!(
                        "Spawn interval: {:.2}s  ({:.2} \u{2192} {:.2})",
                        current, base, cap
                    ))
                    .color(Color32::WHITE),
                );
            }
            if let Some((current, base, cap)) = laser_cooldown {
                ui.label(
                    egui::RichText::new(format!(
                        "Laser cooldown: {:.3}s  ({:.2} \u{2192} {:.3})",
                        current, base, cap
                    ))
                    .color(Color32::WHITE),
                );
            }
        });
}
