use crate::engine::{
    assets::server::AssetServer,
    instance::Instance,
    model::{ loader::load_model_from_obj_bytes },
    state::context::GpuContext,
};

/// Loads and registers the model asset in the
/// worlds AssetServer
pub fn load_obj(
    name: &str,
    obj_bytes: &[u8],
    material_bytes: &[u8],
    gpu_context: &GpuContext,
    initial_instances: Option<Vec<Instance>>,
    max_instances: usize,
    asset_server: &mut AssetServer
) -> usize {
    let model = load_model_from_obj_bytes(
        obj_bytes,
        material_bytes,
        gpu_context,
        initial_instances,
        max_instances
    );
    let model_id = asset_server.register_model(name, model);
    model_id
}
