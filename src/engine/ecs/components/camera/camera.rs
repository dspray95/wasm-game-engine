use cgmath::{ Deg, InnerSpace, Rad, Vector3 };
use wgpu::util::DeviceExt;

use crate::engine::{
    ecs::components::camera::{
        constants::{ DEFAULT_NEAR, DEFAULT_FAR, DEFAULT_FOV },
        projection::Projection,
        uniform::CameraUniformBuffer,
    },
};

pub struct CameraRenderPassData {
    pub buffer: wgpu::Buffer,
    pub uniform_buffer: CameraUniformBuffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Camera {
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub projection: Projection,
    pub render_pass_data: CameraRenderPassData,
}

impl Camera {
    pub fn new(
        yaw: Rad<f32>,
        pitch: Rad<f32>,
        projection: Projection,
        render_pass_data: CameraRenderPassData
    ) -> Self {
        Self { yaw, pitch, projection, render_pass_data }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            })
        )
    }

    pub fn create_render_pass_data(device: &wgpu::Device) -> CameraRenderPassData {
        let uniform_buffer: CameraUniformBuffer = CameraUniformBuffer::new();
        let buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform_buffer]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        );

        let bind_group_layout = Self::create_bind_group_layout(device);

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
                label: Some("camera_bind_group"),
            })
        );

        CameraRenderPassData {
            buffer,
            uniform_buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn handle_resized(&mut self, width: u32, height: u32) {
        self.projection = Projection::new(
            width,
            height,
            Deg(DEFAULT_FOV),
            DEFAULT_NEAR,
            DEFAULT_FAR
        );
    }

    pub fn build_view_projection_matrix(&self, position: Vector3<f32>) -> cgmath::Matrix4<f32> {
        // Create a vector from camera eye to the view direction,
        // calculated from the pitch and yaw
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let view = cgmath::Matrix4::look_to_rh(
            cgmath::Point3::new(position.x, position.y, position.z),
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y()
        );
        // Warp the scene with a projeciton matrix
        let projeciton = self.projection.calculate_projection_matrix();
        projeciton * view
    }

    pub fn update_position(&mut self, position: Vector3<f32>) {
        self.render_pass_data.uniform_buffer.update_position([position.x, position.y, position.z]);
    }

    pub fn update_view_projeciton(&mut self, position: Vector3<f32>) {
        self.render_pass_data.uniform_buffer.update_view_projeciton(
            self.build_view_projection_matrix(position).into()
        );
    }

    pub fn translate(&mut self, position: Vector3<f32>, queue: &wgpu::Queue) {
        // self.position = point3(self.position.x + x, self.position.y + y, self.position.z + z);
        self.update_view_projeciton(position);
        self.update_position(position);
        queue.write_buffer(
            &self.render_pass_data.buffer,
            0,
            bytemuck::cast_slice(&[self.render_pass_data.uniform_buffer])
        );
    }
}

pub struct SurfaceDimensions {
    pub width: f32,
    pub height: f32,
}
