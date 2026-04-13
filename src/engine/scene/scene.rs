use winit::{ event::ElementState, keyboard::KeyCode };

use crate::engine::{
    camera::camera::Camera,
    ecs::{ system::SystemSchedule, world::World },
    model::{ model::Model, model_registry::ModelRegistry },
    state::context::GpuContext,
};

pub trait Scene {
    fn update(&mut self, delta_time: f32, gpu_context: GpuContext, camera: &mut Camera);
    fn handle_key_event(&mut self, key_code: KeyCode, state: ElementState) -> bool;
    fn models(&self) -> &Vec<Model>;

    /// Called once after the window is ready. Scenes register their models and spawn
    /// their initial entities here. Default is a no-op so scenes that haven't migrated yet
    /// don't need to implement it.
    fn setup_ecs(
        &self,
        _world: &mut World,
        _registry: &mut ModelRegistry,
        _schedule: &mut SystemSchedule,
        _gpu: &GpuContext,
    ) {}
}
