use std::collections::HashSet;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Default, Clone)]
pub struct InputState {
    pressed: HashSet<KeyCode>, // Keys that are being held down
    just_pressed: HashSet<KeyCode>, // Keys that went DOWN this frame
    just_released: HashSet<KeyCode>, // Keys that went UP this frame
}

impl InputState {
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub fn record(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                // winit fires Pressed repeatedly while a key is held — only flag
                // just_pressed on the actual first-edge transition.
                if self.pressed.insert(key) {
                    self.just_pressed.insert(key);
                }
            }
            ElementState::Released => {
                if self.pressed.remove(&key) {
                    self.just_released.insert(key);
                }
            }
        }
    }

    pub fn clear_transient(&mut self) {
        // Called by app state before the ECS systems run
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_marks_pressed_and_just_pressed() {
        let mut input = InputState::default();
        input.record(KeyCode::KeyA, ElementState::Pressed);
        assert!(input.is_pressed(KeyCode::KeyA));
        assert!(input.just_pressed(KeyCode::KeyA));
        assert!(!input.just_released(KeyCode::KeyA));
    }

    #[test]
    fn held_key_does_not_re_trigger_just_pressed() {
        let mut input = InputState::default();
        input.record(KeyCode::KeyA, ElementState::Pressed);
        input.clear_transient();
        input.record(KeyCode::KeyA, ElementState::Pressed);
        assert!(input.is_pressed(KeyCode::KeyA));
        assert!(!input.just_pressed(KeyCode::KeyA));
    }

    #[test]
    fn release_clears_pressed_and_marks_just_released() {
        let mut input = InputState::default();
        input.record(KeyCode::KeyA, ElementState::Pressed);
        input.clear_transient();
        input.record(KeyCode::KeyA, ElementState::Released);
        assert!(!input.is_pressed(KeyCode::KeyA));
        assert!(input.just_released(KeyCode::KeyA));
    }

    #[test]
    fn clear_transient_leaves_pressed_intact() {
        let mut input = InputState::default();
        input.record(KeyCode::KeyA, ElementState::Pressed);
        input.clear_transient();
        assert!(input.is_pressed(KeyCode::KeyA));
        assert!(!input.just_pressed(KeyCode::KeyA));
    }

    #[test]
    fn release_of_unpressed_key_is_noop() {
        let mut input = InputState::default();
        input.record(KeyCode::KeyA, ElementState::Released);
        assert!(!input.is_pressed(KeyCode::KeyA));
        assert!(!input.just_released(KeyCode::KeyA));
    }
}
