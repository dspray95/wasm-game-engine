use crate::engine::{ ecs::system::SystemSchedule, ui::ui_registry::UIRegistry };

pub trait Scene {
    fn setup_ecs(&self, _schedule: &mut SystemSchedule) {}
    fn setup_ui(&self, _ui_registry: &mut UIRegistry) {}
}
