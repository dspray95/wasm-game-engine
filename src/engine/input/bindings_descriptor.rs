use std::collections::HashMap;
use std::hash::Hash;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::engine::input::input_state::Binding;

#[derive(Deserialize)]
#[serde(bound = "A: DeserializeOwned + Hash + Eq")]
pub struct BindingsDescriptor<A: Hash + Eq> {
    pub bindings: HashMap<A, Vec<Binding>>,
}
