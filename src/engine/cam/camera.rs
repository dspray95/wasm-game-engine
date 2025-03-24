use cgmath::{ Angle, Deg, InnerSpace, Matrix4, Point3, Rad, Vector3 };
use wgpu::util::DeviceExt;

use super::{ projection::Projection, uniform::CameraUniformBuffer };

pub(in crate::engine) struct CameraRenderPassData {
    pub(in crate::engine) buffer: wgpu::Buffer,
    pub(in crate::engine) uniform_buffer: CameraUniformBuffer,
    pub(in crate::engine) bind_group: wgpu::BindGroup,
    pub(in crate::engine) bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Camera {
    pub position: Point3<f32>,
    pub(super) yaw: Rad<f32>,
    pub(super) pitch: Rad<f32>,
    pub(in crate::engine) projection: Projection,
    pub(in crate::engine) render_pass_data: CameraRenderPassData,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        surface_width: u32,
        surface_height: u32,
        device: &wgpu::Device
    ) -> Self {
        let uniform_buffer: CameraUniformBuffer = CameraUniformBuffer::new();
        let buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform_buffer]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        );

        let bind_group_layout = device.create_bind_group_layout(
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

        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection: Projection::new(surface_width, surface_height, Deg(45.0), 0.1, 100.0),
            render_pass_data: CameraRenderPassData {
                buffer,
                uniform_buffer,
                bind_group,
                bind_group_layout,
            },
        }
    }

    pub fn calculate_view_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y()
        )
    }

    pub fn update_view_projeciton(&mut self) {
        self.render_pass_data.uniform_buffer.update_view_projeciton(
            self.position.to_homogeneous(),
            self.calculate_view_matrix(),
            self.projection.calculate_projection_matrix()
        );
    }
}
