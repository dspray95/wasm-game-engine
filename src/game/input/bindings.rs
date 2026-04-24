use std::collections::HashMap;
use std::hash::Hash;

use crate::engine::input::{
    bindings_descriptor::BindingsDescriptor,
    input_state::{Binding, InputState},
};

pub struct Bindings<A: Hash + Eq> {
    map: HashMap<A, Vec<Binding>>,
}

impl<A: Hash + Eq> Bindings<A> {
    pub fn from_descriptor(descriptor: BindingsDescriptor<A>) -> Self {
        Self { map: descriptor.bindings }
    }

    pub fn is_action_pressed(&self, action: &A, input: &InputState) -> bool {
        self.map
            .get(action)
            .map(|bindings| bindings.iter().any(|b| {
                input.is_pressed(b.key) && input.active_modifiers == b.modifiers
            }))
            .unwrap_or(false)
    }

    pub fn is_action_just_pressed(&self, action: &A, input: &InputState) -> bool {
        self.map
            .get(action)
            .map(|bindings| bindings.iter().any(|b| {
                input.just_pressed_set().contains(&b.key) && input.active_modifiers == b.modifiers
            }))
            .unwrap_or(false)
    }

    pub fn is_action_just_released(&self, action: &A, input: &InputState) -> bool {
        self.map
            .get(action)
            .map(|bindings| bindings.iter().any(|b| {
                input.just_released_set().contains(&b.key)
            }))
            .unwrap_or(false)
    }
}
