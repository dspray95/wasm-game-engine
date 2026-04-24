use std::collections::HashMap;
use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::engine::ecs::{ entity::Entity, world::World };

pub struct ComponentRegistry {
    deserializers: HashMap<String, Box<dyn Fn(&mut World, Entity, &mut dyn erased_serde::Deserializer) -> Result<()>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self { deserializers: HashMap::new() }
    }

    pub fn register<T: DeserializeOwned + 'static>(&mut self, name: &str) {
        self.deserializers.insert(name.to_string(), Box::new(|world, entity, d| {
            let component: T = erased_serde::deserialize(d)?;
            world.add_component(entity, component);
            Ok(())
        }));
    }

    pub fn get(&self, name: &str) -> Option<&(dyn Fn(&mut World, Entity, &mut dyn erased_serde::Deserializer) -> Result<()>)> {
        self.deserializers.get(name).map(|boxed| boxed.as_ref())
    }
}
