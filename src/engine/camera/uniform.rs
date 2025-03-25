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
