use winit::{ event::{ ElementState, KeyEvent, WindowEvent }, keyboard::{ KeyCode, PhysicalKey } };

use crate::engine::camera::Camera;

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
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowRight => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowLeft => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::KeyE => {
                self.is_rotate_right_pressed = is_pressed;
                true
            }
            KeyCode::KeyQ => {
                self.is_rotate_left_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye -= forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye += forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);
        if self.is_left_pressed {
            camera.eye -= right.normalize() * self.speed;
        }
        if self.is_right_pressed {
            camera.eye += right.normalize() * self.speed;
        }

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_rotate_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_rotate_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
