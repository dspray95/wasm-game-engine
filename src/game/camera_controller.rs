use crate::engine::camera::Camera;

const CAMERA_SPEED: f32 = 0.0; //0.0002 looks okayish
pub struct CameraController {
    speed: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: CAMERA_SPEED,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_normalized = forward.normalize();
        camera.eye += forward_normalized * self.speed;
        println!("new camere eye: {:?}", camera.eye);
    }
}
