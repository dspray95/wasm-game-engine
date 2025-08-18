use std::vec;

use crate::{
    engine::{
        model::{ material::Material, model::Model, vertex::ModelVertex },
        resources,
        state::context::GpuContext,
    },
    game::{ cube, starfighter::Starfighter, terrain::Terrain },
};

pub struct SceneManager {
    pub models: Vec<Model>,
}

impl SceneManager {
    pub async fn new<'a>(gpu_context: GpuContext<'a>) -> Self {
        SceneManager::load_scene(gpu_context).await.unwrap()
    }

    // Will load the default scene
    pub async fn load_scene<'a>(gpu_context: GpuContext<'a>) -> anyhow::Result<Self> {
        let terrain_object = Terrain::new(50, 5000);

        let terrain_model = resources::load_model_from_arrays(
            "terrain",
            terrain_object.vertices,
            vec![],
            terrain_object.triangles,
            &gpu_context,
            Material::new([60, 66, 98], 0.5)
        );
        let canyon_model = resources::load_model_from_arrays(
            "canyon",
            terrain_object.canyon_vertices,
            vec![],
            terrain_object.canyon_triangles,
            &gpu_context,
            Material::new([236, 95, 255], 1.0)
        );

        let mut starfighter = resources
            ::load_model_from_file("starfighter.obj", &gpu_context.device).await
            .unwrap();

        for (i, mesh) in starfighter.meshes.iter_mut().enumerate() {
            mesh.position(0.0, 5.0, 5.0, &gpu_context);
        }
        let models: Vec<Model> = vec![canyon_model, terrain_model, starfighter];

        Ok(SceneManager { models })
    }
}
