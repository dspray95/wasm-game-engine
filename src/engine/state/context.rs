use crate::engine::camera::camera::Camera;

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
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub light_bind_group: &'a wgpu::BindGroup,
    pub render_pipeline: &'a wgpu::RenderPipeline,
    pub wireframe_render_pipeline: &'a wgpu::RenderPipeline,
    pub msaa_texture_view: &'a wgpu::TextureView,
    pub msaa_depth_texture_view: &'a wgpu::TextureView,
}
