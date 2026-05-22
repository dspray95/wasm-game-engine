use egui_styled::prelude::*;

use crate::game::{resources::high_scores::MAX_ENTRIES, ui::theme::CanyonColors};

pub fn draw(ui: &mut egui::Ui, entries: &[(String, i32)], submitted_index: Option<usize>) {
    let theme = ui.ctx().styled_theme();
    let colors = ui.ctx().design_data::<CanyonColors>();
    let title_font = theme.font_display(theme.font_size_md);
    let row_font = theme.font_display(theme.font_size_sm);

    Styled::label("HIGH SCORES")
        .font(title_font)
        .text_color(colors.hud_cyan)
        .margin_bottom(theme.spacing_sm)
        .show(ui);

    for slot in 0..MAX_ENTRIES {
        let (initials, score) = entries
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ("---".to_string(), 0));
        let color = if submitted_index == Some(slot) {
            colors.highlight_gold
        } else if entries.get(slot).is_some() {
            colors.text
        } else {
            colors.text_muted
        };
        Styled::label(format!("{:>2}.  {}   {:09}", slot + 1, initials, score))
            .font(row_font.clone())
            .text_color(color)
            .show(ui);
    }
}
