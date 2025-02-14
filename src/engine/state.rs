use wgpu::util::DeviceExt;
use winit::window::Window;
use cgmath::Zero;
use super::{
    camera::{ Camera, CameraUniformBuffer },
    instance::{ Instance, InstanceRaw },
    model::{ self, Model, Vertex },
    resources,
    texture::{ self, Texture },
};
use cgmath::InnerSpace;
use cgmath::Rotation3;
use crate::{
    engine::{ light::LightUniform, render_pipeline::{ self, create_render_pipeline } },
    game::camera_controller::CameraController,
};

const N_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    (N_INSTANCES_PER_ROW as f32) * 0.5,
    0.0,
    (N_INSTANCES_PER_ROW as f32) * 0.5
);

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub render_pipeline: wgpu::RenderPipeline,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: texture::Texture,
    pub camera: Camera,
    pub camera_uniform_buffer: CameraUniformBuffer,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_controller: CameraController,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
    pub depth_texture: Texture,
    pub obj_model: Model,
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_render_pipeline: wgpu::RenderPipeline,
}

impl AppState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        _window: &Window,
        width: u32,
        height: u32
    ) -> Self {
        let scale_factor = 1.0;

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

        // Texture Rendering Setup //
        let diffuse_bytes = include_bytes!("../../happy-tree.png");
        let diffuse_texture = texture::Texture
            ::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png")
            .unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            })
        );

        // Bind group is a more specific declaration of the bind group layout.
        // This allows for swapping out bind groups on the fly (as long as they have the
        // same layout).
        let diffuse_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("diffuse_bind_group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
            })
        );

        // Camera Setup //
        let camera = Camera {
            eye: (0.0, 1.0, -20.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: (surface_config.width as f32) / (surface_config.height as f32),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform_buffer = CameraUniformBuffer::new();
        camera_uniform_buffer.update_view_projeciton(&camera);

        let camera_buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform_buffer]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX, // We only need the camera info in the vertex shader
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false, // We don't change the location of the buffer
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            })
        );

        let camera_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                ],
                label: Some("camera_bind_group"),
            })
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
            color: [1.0, 1.0, 1.0],
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

        // Render Pipeline Definition //
        let render_pipeline_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            })
        );

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        };

        let render_pipeline: wgpu::RenderPipeline = create_render_pipeline(
            &device,
            &render_pipeline_layout,
            surface_config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), InstanceRaw::desc()],
            shader
        );

        // Light Cube Render Pipeline //
        let light_render_pipeline_layout = device.create_pipeline_layout(
            &(wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            })
        );

        let light_shader = wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../light.wgsl").into()),
        };

        let light_render_pipeline = create_render_pipeline(
            &device,
            &light_render_pipeline_layout,
            surface_config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc()],
            light_shader
        );
        // Mesh Rendering //
        const SPACE_BETWEEN: f32 = 3.0;
        const NUM_INSTANCES_PER_ROW: i32 = 1;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * ((x as f32) - (NUM_INSTANCES_PER_ROW as f32) / 2.0);
                    let z = SPACE_BETWEEN * ((z as f32) - (NUM_INSTANCES_PER_ROW as f32) / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = cgmath::Quaternion::from_axis_angle(
                        (0.0, 1.0, 1.0).into(),
                        cgmath::Deg(75.0)
                    );

                    Instance {
                        position,
                        rotation,
                    }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            })
        );

        let obj_model = resources
            ::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await
            .unwrap();

        Self {
            device,
            queue,
            surface,
            surface_config,
            scale_factor,
            render_pipeline,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_uniform_buffer,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_bind_group_layout,
            light_render_pipeline,
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
