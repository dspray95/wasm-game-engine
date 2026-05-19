use cgmath::Vector3;

use crate::engine::ecs::commands::commands::Commands;

pub const DEFAULT_LIFETIME: f32 = 2.5;
pub const DEFAULT_FADE_IN: f32 = 0.15;
pub const DEFAULT_FADE_OUT: f32 = 0.4;
pub const DEFAULT_MAX_VISIBLE_HUD: usize = 4;

/// Anchors a `Toast` to a screen location.
///
/// `HudStack` toasts share a vertical stack below the game's HUD anchor.
/// `World(pos)` toasts attach to a world-space point — used for the future
/// "+100 over a killed enemy" diegetic case; the panel projects them via
/// `engine::ui::projection::world_to_screen` at render time.
#[derive(Clone)]
pub enum ToastAnchor {
    HudStack,
    World(Vector3<f32>),
}

pub struct Toast {
    pub text: String,
    pub elapsed: f32,
    pub lifetime: f32,
    pub fade_in_seconds: f32,
    pub fade_out_seconds: f32,
    pub anchor: ToastAnchor,
}

impl Toast {
    /// Convenience constructor for a HUD-stacked toast with default timings.
    pub fn hud(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            elapsed: 0.0,
            lifetime: DEFAULT_LIFETIME,
            fade_in_seconds: DEFAULT_FADE_IN,
            fade_out_seconds: DEFAULT_FADE_OUT,
            anchor: ToastAnchor::HudStack,
        }
    }

    /// 0.0 → 1.0, sampled by the rendering panel each frame.
    pub fn alpha(&self) -> f32 {
        if self.elapsed < self.fade_in_seconds && self.fade_in_seconds > 0.0 {
            return (self.elapsed / self.fade_in_seconds).clamp(0.0, 1.0);
        }
        let fade_out_start = self.lifetime - self.fade_out_seconds;
        if self.elapsed > fade_out_start && self.fade_out_seconds > 0.0 {
            return ((self.lifetime - self.elapsed) / self.fade_out_seconds).clamp(0.0, 1.0);
        }
        1.0
    }
}

pub struct ToastQueue {
    pub toasts: Vec<Toast>,
    pub max_visible_hud: usize,
}

impl ToastQueue {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_visible_hud: DEFAULT_MAX_VISIBLE_HUD,
        }
    }

    /// Append a toast. If pushing a `HudStack` toast brings the HUD stack
    /// over `max_visible_hud`, force the oldest HUD toast into a quick
    /// fade-out (rather than a hard pop) so the stack settles smoothly.
    pub fn push(&mut self, toast: Toast) {
        let is_hud = matches!(toast.anchor, ToastAnchor::HudStack);
        self.toasts.push(toast);

        if !is_hud {
            return;
        }

        let hud_count = self
            .toasts
            .iter()
            .filter(|t| matches!(t.anchor, ToastAnchor::HudStack))
            .count();
        if hud_count <= self.max_visible_hud {
            return;
        }

        // Find the oldest HUD toast and shrink its remaining lifetime to a
        // quick fade-out. Newest-first sort would do, but we need the index
        // back to mutate in place.
        let oldest_index = self
            .toasts
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(t.anchor, ToastAnchor::HudStack))
            .max_by(|(_, a), (_, b)| {
                a.elapsed
                    .partial_cmp(&b.elapsed)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);

        if let Some(idx) = oldest_index {
            let toast = &mut self.toasts[idx];
            // Force a fade-out: elapsed jumps to (lifetime - fade_out_seconds)
            // unless it's already further along. Capped so this never
            // *extends* the toast's life.
            let target = (toast.lifetime - toast.fade_out_seconds).max(0.0);
            if toast.elapsed < target {
                toast.elapsed = target;
            }
        }
    }
}

/// Queue a `Toast::hud(text)` push via the standard commands buffer. Lets
/// game systems write `push_hud_toast(commands, "SHIELD ENABLED")` instead
/// of constructing the closure boilerplate themselves.
pub fn push_hud_toast(commands: &mut Commands, text: impl Into<String>) {
    let text = text.into();
    commands.update_resource::<ToastQueue, _>(move |queue| {
        queue.push(Toast::hud(text));
    });
}
