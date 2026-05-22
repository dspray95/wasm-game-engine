use egui::{Color32, FontFamily};
use egui_styled::theme::StyledTheme;

/// Canyon Runner geometry + typography tokens. Only deltas from
/// `StyledTheme::default()` are listed — anything not mentioned uses the
/// library default. Colors live in [`CanyonColors`].
pub fn canyon_runner() -> StyledTheme {
    StyledTheme {
        // Tighter type than the default (12/14/18/24) — the pixel font reads
        // chunkier per-em, and the arcade aesthetic wants smaller body text.
        font_size_sm: 11.0,
        font_size_md: 12.0,
        font_size_xl: 28.0,

        // `display` is the pixel font registered in
        // `engine::ui::egui_state::EguiState::new` as `FontFamily::Name("display")`.
        font_family_display: FontFamily::Name("display".into()),

        ..Default::default()
    }
}

/// Canyon Runner's color vocabulary. Names describe the *role* the color
/// plays in this game, not generic web/SaaS slots like "accent" or
/// "warning". Stored on the egui context via
/// `ctx.set_design_data(canyon_colors())`.
#[derive(Clone, Debug, PartialEq)]
pub struct CanyonColors {
    /// Near-black base. Used for the game-over backdrop (with alpha).
    pub background: Color32,
    /// Slightly lifted surface — initials TextEdit fill.
    pub panel_surface: Color32,
    /// Hover-state fill for the SUBMIT button.
    pub panel_elevated: Color32,

    pub text: Color32,
    pub text_muted: Color32,

    /// Signature cyan — HUD titles, SUBMIT button stroke + text.
    pub hud_cyan: Color32,
    /// Lighter cyan for hover states.
    pub hud_cyan_bright: Color32,

    /// Subtle purple field border at rest.
    pub input_border: Color32,
    /// Signature magenta — ENTER INITIALS label, focus glow on the input.
    pub input_magenta: Color32,

    /// Gold — used to highlight the row the player just placed.
    pub highlight_gold: Color32,
    /// Red — input error ("NOT ALLOWED").
    pub danger_red: Color32,
}

impl Default for CanyonColors {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(8, 4, 16),
            panel_surface: Color32::from_rgb(16, 10, 28),
            panel_elevated: Color32::from_rgb(36, 24, 56),

            text: Color32::WHITE,
            text_muted: Color32::from_gray(120),

            hud_cyan: Color32::from_rgb(0, 220, 255),
            hud_cyan_bright: Color32::from_rgb(120, 240, 255),

            input_border: Color32::from_rgb(80, 30, 110),
            input_magenta: Color32::from_rgb(255, 0, 200),

            highlight_gold: Color32::from_rgb(255, 215, 0),
            danger_red: Color32::from_rgb(255, 80, 80),
        }
    }
}

pub fn canyon_colors() -> CanyonColors {
    CanyonColors::default()
}
