use cgmath::{ perspective, Matrix4, Rad };

// This is used to convert the cgmath crate coordinate system to the wgpu system which
// uses normalised device coordinates
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0,  0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0,  0.0, 0.5, 0.5,
    0.0,  0.0, 0.0, 1.0,
);

pub struct Projection {
    aspect: f32,
    fov_y: Rad<f32>,
    z_near: f32,
    z_far: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fov_y: F,
        z_near: f32,
        z_far: f32
    ) -> Self {
        Self {
            aspect: (width as f32) / (height as f32),
            fov_y: fov_y.into(),
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = (width as f32) / (height as f32);
    }

    pub fn calculate_projection_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fov_y, self.aspect, self.z_near, self.z_far)
    }
}
