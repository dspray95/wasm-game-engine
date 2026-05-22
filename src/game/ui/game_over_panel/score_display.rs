use egui_styled::prelude::*;

use crate::game::ui::theme::CanyonColors;

pub fn draw(ui: &mut egui::Ui, final_score: i32) {
    let (theme, colors) = ui.ctx().design::<CanyonColors>();
    Styled::label("YOUR SCORE")
        .font(theme.font_display(theme.font_size_md))
        .text_color(colors.hud_cyan)
        .show(ui);
    Styled::label(format!("{:09}", final_score))
        .font(theme.font_display(theme.font_size_xl))
        .text_color(colors.text)
        .show(ui);
}
