use crate::engine::ecs::world::World;
use crate::engine::ui::ui_panel::UIPanel;

pub struct UIRegistry {
    panels: Vec<UIPanel>,
}

impl UIRegistry {
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }

    pub fn add(&mut self, panel: UIPanel) {
        self.panels.push(panel)
    }

    pub fn draw_all(&self, context: &egui::Context, world: &mut World) {
        for panel in &self.panels {
            panel(context, world);
        }
    }
}
