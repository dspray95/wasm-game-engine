use winit::{ event::ElementState, keyboard::KeyCode };

use crate::engine::{
    camera::camera::Camera,
    ecs::system::SystemSchedule,
    model::model::Model,
    state::context::GpuContext,
};

pub trait Scene {
    fn update(&mut self, delta_time: f32, gpu_context: GpuContext, camera: &mut Camera);
    fn handle_key_event(&mut self, key_code: KeyCode, state: ElementState) -> bool;
    fn models(&self) -> &Vec<Model>;

    /// Called once after the window is ready. Scenes register their startup and update
    /// systems here. Entity spawning and asset loading happen inside the startup systems
    /// themselves via SystemContext. Default is a no-op.
    fn setup_ecs(&self, _schedule: &mut SystemSchedule) {}
}
