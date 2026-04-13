use winit::{ event::ElementState, keyboard::KeyCode };

use crate::engine::{
    camera::camera::Camera,
    model::model::Model,
    state::context::GpuContext,
};

pub trait Scene {
    fn update(&mut self, delta_time: f32, gpu_context: GpuContext, camera: &mut Camera);
    fn handle_key_event(&mut self, key_code: KeyCode, state: ElementState) -> bool;
    fn models(&self) -> &Vec<Model>;
}
