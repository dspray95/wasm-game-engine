use wgpu::util::DeviceExt;

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

pub(super) struct CameraRenderPassData {
    pub(super) buffer: wgpu::Buffer,
    pub(super) uniform_buffer: CameraUniformBuffer,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub(super) render_pass_data: CameraRenderPassData,
}

impl Camera {
    pub fn new(
        location: [f32; 3],
        surface_width: f32,
        surface_height: f32,
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
            eye: (location[0], location[1], location[2]).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_width / surface_height,
            fovy: DEFAULT_FOV,
            znear: DEAFAULT_NEAR,
            zfar: DEFAULT_FAR,
            render_pass_data: CameraRenderPassData {
                buffer,
                uniform_buffer,
                bind_group,
                bind_group_layout,
            },
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Move the world to the position and rotation of the camera
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // Warp the scene with a projeciton matrix
        let projeciton = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect,
            self.znear,
            self.zfar
        );
        OPENGL_TO_WGPU_MATRIX * projeciton * view
    }

    pub fn update_view_projeciton(&mut self) {
        self.render_pass_data.uniform_buffer.update_view_projeciton(
            self.build_view_projection_matrix().into()
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniformBuffer {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniformBuffer {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_projeciton(&mut self, view_projection_matrix: [[f32; 4]; 4]) {
        self.view_projection = view_projection_matrix;
    }
}
