pub fn display(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(egui::FontId::new(
        size,
        egui::FontFamily::Name("display".into()),
    ))
}
