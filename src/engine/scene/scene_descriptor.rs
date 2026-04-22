use std::collections::HashMap;
use anyhow::Result;

use serde::Deserialize;

use crate::engine::{
    assets::server::AssetServer,
    ecs::{
        component_registry::ComponentRegistry,
        components::renderable::Renderable,
        world::World,
    },
};

const RENDERABLE_NAME: &str = "Renderable";

#[derive(Deserialize)]
pub struct SceneDescriptor {
    pub entities: Vec<HashMap<String, ron::Value>>,
}

pub fn load_scene(
    ron_str: &str,
    world: &mut World,
    registry: &ComponentRegistry,
    asset_server: &AssetServer
) -> Result<()> {
    let descriptor: SceneDescriptor = ron::from_str(ron_str)?;

    for entity_data in &descriptor.entities {
        let entity = world.spawn_entity_only();
        for (name, value) in entity_data {
            if name == RENDERABLE_NAME {
                // Special case here, resolve model name into an ID via the asset_server
                #[derive(serde::Deserialize)]
                struct RenderableDescriptor {
                    model: String,
                }
                let descriptor: RenderableDescriptor = value
                    .clone()
                    .into_rust()
                    .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                let model_id = asset_server.get_model_id(&descriptor.model);
                world.add_component(entity, Renderable { model_id });
            } else {
                registry.deserialize_and_insert(world, entity, name, value.clone())?;
            }
        }
    }
    Ok(())
}
