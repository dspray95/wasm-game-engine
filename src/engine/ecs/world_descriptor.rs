use std::fmt;

use anyhow::Result;
use serde::de::{ self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor };

use crate::engine::{
    assets::server::AssetServer,
    ecs::{
        component_registry::ComponentRegistry,
        components::renderable::Renderable,
        entity::Entity,
        world::World,
    },
};

const RENDERABLE_NAME: &str = "Renderable";

// --- Top level: deserializes the `( entities: [ ... ] )` wrapper struct ---

struct WorldDescriptorSeed<'a> {
    world: &'a mut World,
    registry: &'a ComponentRegistry,
    asset_server: &'a AssetServer,
}

impl<'de, 'a> DeserializeSeed<'de> for WorldDescriptorSeed<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        const FIELDS: &[&str] = &["entities"];
        deserializer.deserialize_struct("WorldDescriptor", FIELDS, self)
    }
}

impl<'de, 'a> Visitor<'de> for WorldDescriptorSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a world descriptor with an 'entities' field")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut found_entities = false;

        while let Some(key) = map.next_key::<String>()? {
            if key == "entities" {
                if found_entities {
                    return Err(de::Error::duplicate_field("entities"));
                }
                found_entities = true;
                map.next_value_seed(EntityListSeed {
                    world: self.world,
                    registry: self.registry,
                    asset_server: self.asset_server,
                })?;
            } else {
                return Err(de::Error::unknown_field(&key, &["entities"]));
            }
        }

        if !found_entities {
            return Err(de::Error::missing_field("entities"));
        }

        Ok(())
    }
}

// --- Entity list: deserializes `[ { ... }, { ... } ]` ---

struct EntityListSeed<'a> {
    world: &'a mut World,
    registry: &'a ComponentRegistry,
    asset_server: &'a AssetServer,
}

impl<'de, 'a> DeserializeSeed<'de> for EntityListSeed<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntityListSeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list of entities")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<(), S::Error> {
        while
            seq
                .next_element_seed(EntitySeed {
                    world: self.world,
                    registry: self.registry,
                    asset_server: self.asset_server,
                })?
                .is_some()
        {}

        Ok(())
    }
}

// --- Single entity: deserializes `{ "Transform": (...), "Renderable": (...) }` ---

struct EntitySeed<'a> {
    world: &'a mut World,
    registry: &'a ComponentRegistry,
    asset_server: &'a AssetServer,
}

impl<'de, 'a> DeserializeSeed<'de> for EntitySeed<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'a> Visitor<'de> for EntitySeed<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map of component name to component data")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let entity = self.world.spawn_entity_only();

        while let Some(component_name) = map.next_key::<String>()? {
            if component_name == RENDERABLE_NAME {
                #[derive(serde::Deserialize)]
                struct RenderableDescriptor {
                    model: String,
                }
                let descriptor: RenderableDescriptor = map.next_value()?;
                let model_id = self.asset_server.get_model_id(&descriptor.model);
                self.world.add_component(entity, Renderable::new(model_id));
            } else {
                map.next_value_seed(ComponentSeed {
                    world: self.world,
                    entity,
                    component_name: &component_name,
                    registry: self.registry,
                })?;
            }
        }

        Ok(())
    }
}


// --- Single component value: dispatches through the registry ---

struct ComponentSeed<'a> {
    world: &'a mut World,
    entity: Entity,
    component_name: &'a str,
    registry: &'a ComponentRegistry,
}

impl<'de, 'a> DeserializeSeed<'de> for ComponentSeed<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        let deserialize_fn = self.registry
            .get(self.component_name)
            .ok_or_else(||
                de::Error::custom(format!("unknown component: {}", self.component_name))
            )?;

        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        deserialize_fn(self.world, self.entity, &mut erased).map_err(de::Error::custom)
    }
}

// --- Public API ---

pub fn load_world(
    ron_str: &str,
    world: &mut World,
    registry: &ComponentRegistry,
    asset_server: &AssetServer
) -> Result<()> {
    let mut deserializer = ron::de::Deserializer::from_str(ron_str)?;
    let seed = WorldDescriptorSeed { world, registry, asset_server };
    seed.deserialize(&mut deserializer)?;
    Ok(())
}
