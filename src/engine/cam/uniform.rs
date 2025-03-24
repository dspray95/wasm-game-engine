use cgmath::{ Matrix4, Vector4 };


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniformBuffer {
    view_position: [f32; 4],
    view_projection: [[f32; 4]; 4],
}

impl CameraUniformBuffer {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_projeciton(
        &mut self,
        camera_hommogenouse_position: Vector4<f32>,
        camera_view_matrix: Matrix4<f32>,
        projection_matrix: Matrix4<f32>
    ) {
        self.view_position = camera_hommogenouse_position.into();
        self.view_projection = (projection_matrix * camera_view_matrix).into();
    }
}
