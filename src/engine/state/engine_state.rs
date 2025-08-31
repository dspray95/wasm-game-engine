use anyhow::Error;
use cgmath::{ Deg };
use wgpu::{ util::DeviceExt };
use winit::window::Window;

use crate::engine::{
    camera::{ camera::Camera, controller::CameraController },
    instance::InstanceRaw,
    light::LightUniform,
    model::vertex::{ ModelVertex, Vertex },
    render_pipeline::{ create_render_pipeline, create_wireframe_render_pipeline },
    state::context::{ GpuContext, RenderContext },
    texture::{ self, Texture },
};

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
    pub msaa_texture: wgpu::Texture,
    pub msaa_texture_view: wgpu::TextureView,
    pub msaa_depth_texture: wgpu::Texture,
    pub msaa_depth_texture_view: wgpu::TextureView,
}

impl EngineState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        _window: &Window,
        width: u32,
        height: u32
    ) -> std::result::Result<EngineState, Error> {
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
        device.on_uncaptured_error(
            Box::new(|error| {
                log::error!("Uncaptured WebGPU device error: {:?}", error);
            })
        );
        // Surface Setup //
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats
            .iter()
            .find(|f| f.is_srgb()) // Prefer sRGB formats
            .or_else(|| swapchain_capabilities.formats.first()) // Fallback to any format
            .expect("No surface formats available!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: swapchain_capabilities.present_modes
                .iter()
                .find(|&&mode| mode == wgpu::PresentMode::Fifo)
                .copied()
                .unwrap_or(swapchain_capabilities.present_modes[0]),
            desired_maximum_frame_latency: 2, // 2 for better WebGL compatibility
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        // Camera + Controller //
        let camera = Camera::new(
            [24.5, -0.25, 1.0],
            Deg(90.0),
            Deg(0.0),
            surface_config.width,
            surface_config.height,
            &device
        );
        let camera_controller = CameraController::new();

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

        let color_bind_group_layout = device.create_bind_group_layout(
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

        // Render Pipeline Definition //
        let render_pipeline_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.render_pass_data.bind_group_layout,
                    &light_bind_group_layout,
                    &color_bind_group_layout,
                ],
                push_constant_ranges: &[],
            })
        );

        // Shader + Pipeline Setup //
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Base Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shader.wgsl").into()),
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
            label: Some("Base Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../wireframe.wgsl").into()),
        };
        let wireframe_render_pipeline: wgpu::RenderPipeline = create_wireframe_render_pipeline(
            &device,
            &render_pipeline_layout,
            surface_config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            wireframe_shader
        );

        // MSAA setup -
        let msaa_texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("MSAA Framebuffer"),
                size: wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[], // Add an empty slice for view_formats
            })
        );
        let msaa_texture_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_format = wgpu::TextureFormat::Depth32Float;

        let msaa_depth_texture = device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("MSAA Depth Texture"),
                size: wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4, // match MSAA color
                dimension: wgpu::TextureDimension::D2,
                format: depth_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        );
        let msaa_depth_texture_view = msaa_depth_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        Ok(Self {
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
            msaa_texture,
            msaa_texture_view,
            msaa_depth_texture,
            msaa_depth_texture_view,
        })
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        // Update the surface
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        // Recreate MSAA color texture and view, we need to do this
        // since the mssaa texture will error if we try to draw it to
        // our resized surface without first changing their width and height
        self.msaa_texture = self.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("MSAA Framebuffer"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        );
        self.msaa_texture_view = self.msaa_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        // Recreate MSAA depth texture and view
        let depth_format = wgpu::TextureFormat::Depth32Float;
        self.msaa_depth_texture = self.device.create_texture(
            &(wgpu::TextureDescriptor {
                label: Some("MSAA Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: depth_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        );
        self.msaa_depth_texture_view = self.msaa_depth_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
    }

    pub(crate) fn gpu_context(&self) -> GpuContext<'_> {
        GpuContext { device: &self.device, queue: &self.queue }
    }

    pub(crate) fn render_context(&self) -> RenderContext<'_> {
        RenderContext {
            device: &self.device,
            queue: &self.queue,
            surface: &self.surface,
            surface_config: &self.surface_config,
            depth_texture_view: &self.depth_texture.view,
            camera_bind_group: &self.camera.render_pass_data.bind_group,
            light_bind_group: &self.light_bind_group,
            render_pipeline: &self.render_pipeline,
            wireframe_render_pipeline: &self.wireframe_render_pipeline,
            msaa_texture_view: &self.msaa_texture_view,
            msaa_depth_texture_view: &self.msaa_depth_texture_view,
        }
    }
}
