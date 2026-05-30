use egui::{Color32, FontFamily};
use egui_styled::prelude::*;
use egui_styled::theme::StyledTheme;

/// Empirical horizontal correction for centered text rendered in the
/// `display` family (`re-do.ttf`). The pixel font's glyphs sit shifted left
/// within their advance cells, so egui's text centring lands the layout box
/// correctly but the visible pixels appear left-of-centre. Adding this
/// value to a target x-position pushes the visible content back toward the
/// intended centre.
///
/// The right long-term fix is to re-export `re-do.ttf` with corrected
/// horizontal bearings (FontForge can do this in ~15 minutes). Until then,
/// this constant is the workaround — apply it wherever the display font
/// is centred on a known point.
pub const DISPLAY_FONT_VISUAL_X_CORRECTION: f32 = 8.0;

/// Canyon Runner geometry + typography tokens. Only deltas from
/// `StyledTheme::default()` are listed — anything not mentioned uses the
/// library default. Colors live in [`CanyonColors`].
pub fn canyon_runner() -> StyledTheme {
    StyledTheme {
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

    /// Bright red — the player's remaining health pips beneath the ship.
    /// Lost pips are derived from this via `.darken().with_alpha()` rather
    /// than being a separate named token; that keeps the two visually
    /// related when the alive color is tuned.
    pub health_alive: Color32,
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

            health_alive: Color32::from_rgb(220, 60, 60),
        }
    }
}

pub fn canyon_colors() -> CanyonColors {
    CanyonColors::default()
}

/// Chromatic-aberration preset: an RGB split painted as two opposite-offset
/// glyph shadows (signature cyan to the left, magenta to the right). `offset`
/// is the horizontal split in pixels — pass a constant for a static split or
/// an animated value for a shimmer. Compose onto any label with `.apply()`:
///
/// ```ignore
/// Styled::label("[ENTER]")
///     .text_color(Color32::WHITE)
///     .apply(chromatic_aberration(&colors, 2.0))
///     .show(ui);
/// ```
pub fn chromatic_aberration(
    colors: &CanyonColors,
    offset: f32,
) -> impl Fn(StyledLabel) -> StyledLabel + 'static {
    let (cyan, magenta) = (colors.hud_cyan, colors.input_magenta);
    move |label| {
        label
            .text_shadow(egui::vec2(-offset, 0.0), cyan)
            .text_shadow(egui::vec2(offset, 0.0), magenta)
    }
}
