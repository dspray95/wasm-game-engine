use std::collections::HashMap;

use crate::engine::{
    ecs::components::collider::{ Collider, ColliderShape },
    instance::Instance,
    model::{ loader::load_model_from_obj_bytes, model::Model, model_registry::{ ModelRegistry } },
    state::context::GpuContext,
};

pub struct AssetServer {
    models: HashMap<String, usize>,
    model_registry: ModelRegistry,
}

impl AssetServer {
    pub fn new() -> Self {
        Self { models: HashMap::new(), model_registry: ModelRegistry::new() }
    }

    pub fn load_model(
        &mut self,
        name: &str,
        obj_bytes: &[u8],
        material_bytes: &[u8],
        gpu_context: &GpuContext,
        initial_instances: Option<Vec<Instance>>,
        max_instances: Option<usize>
    ) -> usize {
        let model = load_model_from_obj_bytes(
            obj_bytes,
            material_bytes,
            gpu_context,
            initial_instances,
            max_instances.unwrap_or(1024)
        );
        self.register_model(name, model)
    }

    pub fn register_model(&mut self, name: &str, model: Model) -> usize {
        let model_id = self.model_registry.register(model);
        self.models.insert(name.to_string(), model_id);
        model_id
    }

    pub fn get_model_id(&self, name: &str) -> usize {
        *self.models.get(name).unwrap_or_else(|| panic!("Asset '{}' not registered", name))
    }

    pub fn models(&self) -> &[Model] {
        self.model_registry.models()
    }

    pub fn get_model(&self, id: usize) -> &Model {
        self.model_registry.get(id).unwrap_or_else(|| panic!("No model registered with id {}", id))
    }

    pub fn get_model_mut(&mut self, id: usize) -> &mut Model {
        self.model_registry
            .get_mut(id)
            .unwrap_or_else(|| panic!("No model registered with id {}", id))
    }

    /// Builds an AABB collider from the model's vertex bounds, in model-local space.
    /// `collision_system` and `collider_debug_system` apply `Transform.scale` at runtime,
    /// so do not pre-scale this before attaching to an entity.
    pub fn get_collider_aabb(&self, name: &str) -> Collider {
        let bounds = self.get_model(self.get_model_id(name)).bounds;
        Collider {
            shape: ColliderShape::AABB {
                offset: bounds.center(),
                half_extents: bounds.half_extents(),
            },
        }
    }
}
