use crate::{
    engine::{
        camera::camera::Camera,
        ecs::{ system::SystemContext, world::World },
        state::context::GpuContext,
    },
    game::{
        helpers::terrain_generation::{ replace_terrain_model_buffers, terrain_update },
        resources::terrain_resources::{ TerrainGeneration, TerrainModelIds },
    },
};

pub fn terrain_system(world: &mut World, system_context: &mut SystemContext) {
    let camera_z = world.get_resource::<Camera>().unwrap().position.z;

    let terrain_result = {
        let terrain_generation = world.get_resource_mut::<TerrainGeneration>().unwrap();
        terrain_update(terrain_generation, camera_z)
    };

    if let Some(new_terrain_mesh_data) = terrain_result {
        let terrain_model_ids: [usize; 3] = world.get_resource::<TerrainModelIds>().unwrap().0;

        let oldest_index = {
            let terrain_generation = world.get_resource_mut::<TerrainGeneration>().unwrap();
            let index = terrain_generation.oldest_terrain_index as usize;
            terrain_generation.oldest_terrain_index =
                (terrain_generation.oldest_terrain_index + 1) % 3;
            index
        };

        let model_to_replace = system_context.asset_server
            .as_mut()
            .unwrap()
            .get_model_mut(terrain_model_ids[oldest_index]);

        replace_terrain_model_buffers(
            new_terrain_mesh_data,
            model_to_replace,
            &(GpuContext {
                device: system_context.device.unwrap(),
                queue: system_context.queue.unwrap(),
            })
        );
    }
}
