#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniformBuffer {
    view_projection: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
}

impl CameraUniformBuffer {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_projection: cgmath::Matrix4::identity().into(),
            position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        }
    }

    pub fn update_position(&mut self, new_position: [f32; 3]) {
        self.position = new_position;
    }

    pub fn update_view_projeciton(&mut self, view_projection_matrix: [[f32; 4]; 4]) {
        self.view_projection = view_projection_matrix;
    }
}
