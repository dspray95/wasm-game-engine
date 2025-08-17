use crate::{
    engine::{ model::model::Model, resources, state::context::GpuContext },
    game::terrain::Terrain,
};

pub struct SceneManager {
    pub models: Vec<Model>,
}

impl SceneManager {
    pub fn new(gpu_context: GpuContext) -> Self {
        SceneManager::load_scene(gpu_context)
    }
    // Will load the default scene
    pub fn load_scene(gpu_context: GpuContext) -> Self {
        let terrain_object = Terrain::new(50, 5000);
        let terrain_model = resources::load_model_from_arrays(
            "terrain",
            terrain_object.vertices,
            vec![],
            terrain_object.triangles,
            &gpu_context,
            [60, 66, 98]
        );
        let canyon_model = resources::load_model_from_arrays(
            "canyon",
            terrain_object.canyon_vertices,
            vec![],
            terrain_object.canyon_triangles,
            &gpu_context,
            [255, 0, 255]
        );
        let models: Vec<Model> = vec![terrain_model, canyon_model];
        SceneManager { models }
    }
}
