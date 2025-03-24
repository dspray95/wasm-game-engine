use std::time::Instant;

use cgmath::Deg;
use wgpu::util::DeviceExt;
use winit::window::Window;
use super::{
    cam::{ camera::Camera, controller::CameraController },
    instance::InstanceRaw,
    model::vertex::{ ModelVertex, Vertex },
    render_pipeline::create_wireframe_render_pipeline,
    texture::{ self, Texture },
};
use crate::{ engine::{ light::LightUniform, render_pipeline::create_render_pipeline } };

pub struct EngineState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub depth_texture: Texture,
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub wireframe_render_pipeline: wgpu::RenderPipeline,
    pub index_bind_group_layout: wgpu::BindGroupLayout,
    pub positions_bind_group_layout: wgpu::BindGroupLayout,
    pub last_redraw_requested_time: Instant,
}

impl EngineState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        _window: &Window,
        width: u32,
        height: u32
    ) -> Self {
        // Device and adapter setup //
        let power_preference = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference,
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
            ).await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                }),
                None
            ).await
            .expect("Failed to create device");

        // Surface Setup //
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities.formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        // Camera + Controller //
        let camera = Camera::new(
            [0.0, 5.0, 10.0],
            Deg(-90.0),
            Deg(-20.0),
            surface_config.width,
            surface_config.height,
            &device
        );
        let camera_controller = CameraController::new(10.0, 10.0);

        // Depth texture //
        let depth_texture = Texture::create_depth_texture(
            &device,
            &surface_config,
            "depth_texture"
        );

        // Global Lighting Setup //
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [0.443, 0.941, 0.922],
            __padding: 0,
        };

        let light_buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        );

        let light_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
        );

        let light_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: None,
                layout: &light_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: light_buffer.as_entire_binding(),
                    },
                ],
            })
        );

        // Bind Group Layouts for Wireframe Renderer //
        // The actual bind groups are defined on a per-mesh basis, built with this layout
        let index_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some(&"index_bind_group_layout"),
            })
        );

        let positions_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some(&"positions_bind_group_layout"),
            })
        );

        // Render Pipeline Definitions //
        let render_pipeline_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.render_pass_data.bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            })
        );

        let render_wireframe_pipeline_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Render Wireframe Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.render_pass_data.bind_group_layout,
                    &index_bind_group_layout,
                    &positions_bind_group_layout,
                ],
                push_constant_ranges: &[],
            })
        );

        // Shader + Pipeline Setup //
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Base Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        };

        let render_pipeline: wgpu::RenderPipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            surface_config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            shader
        );

        let wireframe_shader = wgpu::ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../wireframe.wgsl").into()),
        };
        let wireframe_render_pipeline: wgpu::RenderPipeline = create_wireframe_render_pipeline(
            &device,
            &render_wireframe_pipeline_layout,
            surface_config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            wireframe_shader
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            camera,
            camera_controller,
            depth_texture,
            light_uniform,
            light_buffer,
            light_bind_group,
            wireframe_render_pipeline,
            index_bind_group_layout,
            positions_bind_group_layout,
            last_redraw_requested_time: Instant::now(),
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
