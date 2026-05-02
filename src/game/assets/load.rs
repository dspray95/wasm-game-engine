use cgmath::{ Quaternion, vec3 };

use crate::{
    engine::{
        assets::{ loader::load_obj, server::AssetServer },
        ecs::world::World,
        state::context::GpuContext,
    },
    game::{
        assets::include::{
            CUBE_PREFAB_MTL,
            CUBE_PREFAB_OBJ,
            LASER_MODEL_MTL,
            LASER_MODEL_OBJ,
            STARFIGHTER_ENEMY_MTL,
            STARFIGHTER_MODEL_OBJ,
            STARFIGHTER_PLAYER_MTL,
        },
    },
};

pub fn load_and_register_world_models(
    gpu_context: &GpuContext,
    asset_server: &mut AssetServer,
    _world: &mut World
) {
    // Player
    load_obj(
        "starfighter",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_PLAYER_MTL,
        &gpu_context,
        None,
        1,
        asset_server
    );

    // Enemy
    load_obj(
        "starfighter_enemy",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_ENEMY_MTL,
        &gpu_context,
        None,
        50,
        asset_server
    );
    // Cube
    load_obj("cube", CUBE_PREFAB_OBJ, CUBE_PREFAB_MTL, &gpu_context, None, 64, asset_server);

    // Laser
    load_obj("laser", LASER_MODEL_OBJ, LASER_MODEL_MTL, gpu_context, None, 1024, asset_server);
}
