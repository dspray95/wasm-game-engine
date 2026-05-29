use std::f32::consts::TAU;

use egui::Color32;
use egui_styled::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::ui::theme::CanyonColors;

const ROLL_DURATION_SECONDS: f32 = 1.0;
const SCORE_DIGITS: usize = 9;
const SCORE_FOR_MAX_GLITCH: f32 = 20_000.0;
const GLITCH_MIN_FACTOR: f32 = 0.25;
const GLITCH_MAX_OFFSET_PIXELS: f32 = 5.0;
const GLITCH_MIN_DURATION_SECONDS: f32 = 0.5;
const GLITCH_MAX_DURATION_SECONDS: f32 = 2.0;
const GLITCH_DIRECTION_INTERVAL_SECONDS: f32 = 0.01;
const GLITCH_REPLAY_MIN_INTERVAL_SECONDS: f64 = 2.0;
const GLITCH_REPLAY_MAX_INTERVAL_SECONDS: f64 = 6.0;
const GLITCH_CYAN: Color32 = Color32::from_rgb(0, 220, 255);
const GLITCH_MAGENTA: Color32 = Color32::from_rgb(255, 0, 200);

/// Persisted in egui data between frames to drive periodic replay glitches.
#[derive(Clone, Default)]
struct GlitchReplayState {
    /// egui time when the next replay should begin.
    next_glitch_at: f64,
    /// egui time when the current replay started (0.0 = none active yet).
    replay_start: f64,
}

pub fn draw(ui: &mut egui::Ui, _final_score: i32, reveal_elapsed: f32) {
    let final_score = 20_000;
    let (theme, colors) = ui.ctx().design::<CanyonColors>();

    Styled::label("YOUR SCORE")
        .font(theme.font_display(theme.font_size_md))
        .text_color(colors.hud_cyan)
        .show(ui);

    let score_factor = (final_score as f32 / SCORE_FOR_MAX_GLITCH)
        .clamp(0.0, 1.0)
        .max(GLITCH_MIN_FACTOR);
    let glitch_duration = GLITCH_MIN_DURATION_SECONDS
        + (GLITCH_MAX_DURATION_SECONDS - GLITCH_MIN_DURATION_SECONDS) * score_factor;
    let animation_end = ROLL_DURATION_SECONDS + glitch_duration;

    let now = ui.ctx().input(|i| i.time);
    let replay_id = egui::Id::new("score_display_glitch_replay");

    // During the initial reveal, keep replay state cleared so a fresh reveal
    // always starts clean after a restart.
    if reveal_elapsed < animation_end {
        ui.ctx()
            .data_mut(|d| d.remove::<GlitchReplayState>(replay_id));
    }

    // After the reveal, manage periodic replay glitches.
    let replay_elapsed: Option<f32> = if reveal_elapsed >= animation_end {
        let mut rng = rand::rng();
        let state = ui.ctx().data_mut(|d| {
            let state = d.get_temp_mut_or_default::<GlitchReplayState>(replay_id);
            if now >= state.next_glitch_at && (now - state.replay_start) >= glitch_duration as f64 {
                state.replay_start = now;
                state.next_glitch_at = now
                    + rng.random_range(
                        GLITCH_REPLAY_MIN_INTERVAL_SECONDS..GLITCH_REPLAY_MAX_INTERVAL_SECONDS,
                    );
            }
            state.clone()
        });
        let elapsed = (now - state.replay_start) as f32;
        if elapsed < glitch_duration {
            Some(elapsed)
        } else {
            None
        }
    } else {
        None
    };

    // Initial reveal takes priority; after that, replays take over.
    let glitch_offset = if reveal_elapsed < animation_end {
        compute_glitch_offset(reveal_elapsed, score_factor, glitch_duration)
    } else if let Some(elapsed) = replay_elapsed {
        compute_glitch_offset(ROLL_DURATION_SECONDS + elapsed, score_factor, glitch_duration)
    } else {
        egui::Vec2::ZERO
    };

    let text = build_score_text(final_score, reveal_elapsed);
    let font = theme.font_display(theme.font_size_xl);

    if glitch_offset != egui::Vec2::ZERO {
        Styled::stack()
            .layer_offset(glitch_offset, {
                let (text, font) = (text.clone(), font.clone());
                move |ui| {
                    Styled::label(&text)
                        .font(font)
                        .text_color(GLITCH_CYAN)
                        .show(ui);
                }
            })
            .layer_offset(-glitch_offset, {
                let (text, font) = (text.clone(), font.clone());
                move |ui| {
                    Styled::label(&text)
                        .font(font)
                        .text_color(GLITCH_MAGENTA)
                        .show(ui);
                }
            })
            .layer(|ui| {
                Styled::label(&text)
                    .font(font)
                    .text_color(colors.text)
                    .show(ui);
            })
            .show(ui);
    } else {
        Styled::label(&text)
            .font(font)
            .text_color(colors.text)
            .show(ui);
    }

    if reveal_elapsed < animation_end || replay_elapsed.is_some() {
        ui.ctx().request_repaint();
    }
}

fn compute_glitch_offset(
    reveal_elapsed: f32,
    score_factor: f32,
    glitch_duration: f32,
) -> egui::Vec2 {
    if glitch_duration <= 0.0 {
        return egui::Vec2::ZERO;
    }
    let glitch_elapsed = reveal_elapsed - ROLL_DURATION_SECONDS;
    if glitch_elapsed <= 0.0 || glitch_elapsed >= glitch_duration {
        return egui::Vec2::ZERO;
    }
    let decay = 1.0 - (glitch_elapsed / glitch_duration);
    let magnitude = GLITCH_MAX_OFFSET_PIXELS * score_factor * decay;
    // Deterministic seed per interval tick keeps direction stable within each window.
    let tick = (glitch_elapsed / GLITCH_DIRECTION_INTERVAL_SECONDS) as u64;
    let mut rng = StdRng::seed_from_u64(tick);
    let angle: f32 = rng.random_range(0.0..TAU);
    egui::vec2(angle.cos() * magnitude, angle.sin() * magnitude)
}

fn build_score_text(final_score: i32, reveal_elapsed: f32) -> String {
    if reveal_elapsed >= ROLL_DURATION_SECONDS {
        return format!("{:09}", final_score);
    }
    let progress = reveal_elapsed / ROLL_DURATION_SECONDS;
    let locked_count = (progress * SCORE_DIGITS as f32).floor() as usize;
    let score_str = format!("{:09}", final_score);
    let mut rng = rand::rng();
    score_str
        .chars()
        .enumerate()
        .map(|(index, real_digit)| {
            if index < locked_count {
                real_digit
            } else {
                char::from_digit(rng.random_range(0..10), 10).unwrap()
            }
        })
        .collect()
}
