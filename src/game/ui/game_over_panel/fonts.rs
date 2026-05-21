pub struct PanelFonts {
    pub title: egui::FontId,
    pub score: egui::FontId,
    pub row: egui::FontId,
}

impl PanelFonts {
    pub fn new() -> Self {
        let family = egui::FontFamily::Name("display".into());
        Self {
            title: egui::FontId::new(12.0, family.clone()),
            score: egui::FontId::new(28.0, family.clone()),
            row: egui::FontId::new(11.0, family),
        }
    }
}
