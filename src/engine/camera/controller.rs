use cgmath::{ Rad, Vector3 };
use winit::{ event::{ ElementState, KeyEvent, WindowEvent }, keyboard::{ KeyCode, PhysicalKey } };

use super::camera::Camera;

const CAMERA_SPEED: f32 = -0.2; //0.0002 looks okayish
pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_rotate_left_pressed: bool,
    is_rotate_right_pressed: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: CAMERA_SPEED,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_rotate_left_pressed: false,
            is_rotate_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, key_code: KeyCode, key_state: ElementState) -> bool {
        let is_pressed = key_state == ElementState::Pressed;

        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::KeyQ => {
                self.is_rotate_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyE => {
                self.is_rotate_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        // forward + backward
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        if self.is_forward_pressed {
            camera.position -= forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.position += forward * self.speed;
        }

        // left + right
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        if self.is_left_pressed {
            camera.position += right * self.speed;
        }
        if self.is_right_pressed {
            camera.position -= right * self.speed;
        }

        // rotate
        if self.is_rotate_left_pressed {
            camera.yaw -= Rad(0.025);
        }
        if self.is_rotate_right_pressed {
            camera.yaw += Rad(0.025);
        }
    }
}
