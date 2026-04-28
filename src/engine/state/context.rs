use winit::window::Window;

use crate::engine::ui::egui_state::EguiState;

pub struct GpuContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub surface: &'a wgpu::Surface<'static>,
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    pub depth_texture_view: &'a wgpu::TextureView,
    pub camera_bind_group: Option<&'a wgpu::BindGroup>,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub render_pipeline: &'a wgpu::RenderPipeline,
    pub wireframe_render_pipeline: &'a wgpu::RenderPipeline,
    pub msaa_texture_view: &'a wgpu::TextureView,
    pub msaa_depth_texture_view: &'a wgpu::TextureView,
}

pub struct EguiContext<'a> {
    pub state: &'a mut EguiState,
    pub full_output: egui::FullOutput,
    pub window: &'a Window,
}
