// This is used to convert the cgmath crate coordinate system to the wgpu system which
// uses normalised device coordinates
#[rustfmt::skip]pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0,  0.0, 0.5, 0.5,
    0.0,  0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Move the world to the position and rotation of the camera
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // Warp the scene with a projeciton matrix
        let projeciton =
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * projeciton * view
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

    pub fn update_view_projeciton(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}
