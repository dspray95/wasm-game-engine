use std::collections::HashMap;
use anyhow::Result;

use crate::engine::ecs::{ entity::Entity, world::World };

pub struct ComponentRegistry {
    deserializers: HashMap<String, Box<dyn Fn(&mut World, Entity, ron::Value) -> Result<()>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self { deserializers: HashMap::new() }
    }

    pub fn register<T>(&mut self, name: &str) where T: serde::de::DeserializeOwned + 'static {
        // Registers a type-erased deserializer closure that can later convert a
        // RON value into component T and add it to an entity
        self.deserializers.insert(
            name.to_string(),
            Box::new(|world, entity, value| {
                let component: T = value
                    .into_rust()
                    .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                world.add_component(entity, component);
                Ok(())
            })
        );
    }

    pub fn register_raw(
        &mut self,
        name: &str,
        f: Box<dyn Fn(&mut World, Entity, ron::Value) -> Result<()>>
    ) {
        self.deserializers.insert(name.to_string(), f);
    }

    pub fn deserialize_and_insert(
        &self,
        world: &mut World,
        entity: Entity,
        name: &str,
        value: ron::Value
    ) -> Result<()> {
        let deserializer = self.deserializers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown component: {}", name))?;
        deserializer(world, entity, value)
    }
}
