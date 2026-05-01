use cgmath::{ Quaternion, vec3 };

use crate::{
    engine::{
        assets::{ loader::load_obj, server::AssetServer },
        ecs::world::World,
        instance::Instance,
        state::context::GpuContext,
    },
    game::{
        assets::include::{
            CUBE_PREFAB_MTL,
            CUBE_PREFAB_OBJ,
            LASER_MODEL_MTL,
            LASER_MODEL_OBJ,
            STARFIGHTER_MODEL_MTL,
            STARFIGHTER_MODEL_OBJ,
        },
        helpers::laser::MAX_ALIVE_LASERS,
    },
};

pub fn load_and_register_world_models(
    gpu_context: &GpuContext,
    asset_server: &mut AssetServer,
    world: &mut World
) {
    // Player
    load_obj(
        "starfighter",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_MODEL_MTL,
        &gpu_context,
        None,
        10,
        asset_server
    );

    // Cube
    load_obj("cube", CUBE_PREFAB_OBJ, CUBE_PREFAB_MTL, &gpu_context, None, 64, asset_server);

    // Laser
    let initial_laser_instances: Vec<Instance> = (0..MAX_ALIVE_LASERS as usize)
        .map(|_| Instance {
            position: vec3(0.0, -1000.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: vec3(0.0, 0.0, 0.0),
        })
        .collect();

    load_obj(
        "laser",
        LASER_MODEL_OBJ,
        LASER_MODEL_MTL,
        gpu_context,
        Some(initial_laser_instances),
        MAX_ALIVE_LASERS as usize,
        asset_server
    );
}
