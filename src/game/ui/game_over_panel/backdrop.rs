use egui_styled::prelude::*;

use crate::game::ui::theme::CanyonColors;

const BACKDROP_ALPHA: u8 = 180;

pub fn draw(context: &egui::Context) {
    let colors = context.design_data::<CanyonColors>();
    Styled::area()
        .id("game_over_backdrop")
        .order(egui::Order::Background)
        .fill_screen()
        .bg(colors.background.with_alpha(BACKDROP_ALPHA))
        .show(context, |_| {});
}
