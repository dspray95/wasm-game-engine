use std::collections::HashSet;

use serde::Deserialize;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Modifiers {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
}

#[derive(Clone, Deserialize)]
pub struct Binding {
    pub key: KeyCode,
    #[serde(default)]
    pub modifiers: Modifiers,
}

#[derive(Default, Clone)]
pub struct InputState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
    pub active_modifiers: Modifiers,
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

    pub fn pressed_set(&self) -> &HashSet<KeyCode> {
        &self.pressed
    }

    pub fn just_pressed_set(&self) -> &HashSet<KeyCode> {
        &self.just_pressed
    }

    pub fn just_released_set(&self) -> &HashSet<KeyCode> {
        &self.just_released
    }

    pub fn record(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.pressed.insert(key) {
                    self.just_pressed.insert(key);
                }
                self.update_modifier_state(key, true);
            }
            ElementState::Released => {
                if self.pressed.remove(&key) {
                    self.just_released.insert(key);
                }
                self.update_modifier_state(key, false);
            }
        }
    }

    fn update_modifier_state(&mut self, key: KeyCode, is_pressed: bool) {
        match key {
            KeyCode::ControlLeft | KeyCode::ControlRight => self.active_modifiers.ctrl = is_pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.active_modifiers.shift = is_pressed,
            KeyCode::AltLeft | KeyCode::AltRight => self.active_modifiers.alt = is_pressed,
            _ => {}
        }
    }

    pub fn clear_transient(&mut self) {
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

    #[test]
    fn ctrl_modifier_tracked_on_press_and_release() {
        let mut input = InputState::default();
        input.record(KeyCode::ControlLeft, ElementState::Pressed);
        assert!(input.active_modifiers.ctrl);
        input.record(KeyCode::ControlLeft, ElementState::Released);
        assert!(!input.active_modifiers.ctrl);
    }
}
