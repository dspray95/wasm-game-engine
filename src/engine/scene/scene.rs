use crate::engine::{
    camera::camera::Camera,
    ecs::{resources::input_state::InputState, system::SystemSchedule},
    model::model::Model,
    state::context::GpuContext,
};

pub trait Scene {
    fn update(&mut self, _delta_time: f32, _gpu_context: GpuContext, _camera: &mut Camera, _input: &InputState) {}
    fn models(&self) -> &[Model] { &[] }
    fn setup_ecs(&self, _schedule: &mut SystemSchedule) {}
}
