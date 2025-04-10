use cgmath::{ Deg, InnerSpace, Point3, Rad, Vector3 };
use wgpu::util::DeviceExt;

use super::{ projection::Projection, uniform::CameraUniformBuffer };

const DEFAULT_FOV: f32 = 75.0;
const DEAFAULT_NEAR: f32 = 0.1;
const DEFAULT_FAR: f32 = 100.0;

// This is used to convert the cgmath crate coordinate system to the wgpu system which
// uses normalised device coordinates
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0,  0.0, 0.5, 0.5,
    0.0,  0.0, 0.0, 1.0,
);

pub struct CameraRenderPassData {
    pub buffer: wgpu::Buffer,
    pub uniform_buffer: CameraUniformBuffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Camera {
    pub position: Point3<f32>,
    pub(super) yaw: Rad<f32>,
    pub(super) pitch: Rad<f32>,
    pub projection: Projection,
    pub render_pass_data: CameraRenderPassData,
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

        Camera {
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

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Create a vector from camera eye to the view direction,
        // calculated from the pitch and yaw
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let view = cgmath::Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y()
        );
        // Warp the scene with a projeciton matrix
        let projeciton = self.projection.calculate_projection_matrix();
        projeciton * view
    }

    pub fn update_view_projeciton(&mut self) {
        self.render_pass_data.uniform_buffer.update_view_projeciton(
            self.build_view_projection_matrix().into()
        );
    }
}
