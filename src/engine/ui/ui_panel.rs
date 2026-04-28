use crate::engine::ecs::world::World;

pub type UIPanel = fn(&egui::Context, &mut World);
