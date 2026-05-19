use crate::{
    engine::{
        assets::{loader::load_obj, server::AssetServer},
        ecs::world::World,
        state::context::GpuContext,
    },
    game::assets::include::{
        CUBE_PREFAB_MTL, CUBE_PREFAB_OBJ, EXPLOSION_MODEL_MTL, EXPLOSION_MODEL_OBJ,
        LASER_GLITCH_CYAN_MTL, LASER_GLITCH_WHITE_MTL, LASER_MODEL_MTL, LASER_MODEL_OBJ,
        PICKUP_GLITCH_CYAN_MTL, PICKUP_MODEL_MTL, PICKUP_MODEL_OBJ, PICKUP_WHITE_MTL,
        SHIELD_MODEL_MTL, SHIELD_MODEL_OBJ,
        STARFIGHTER_ENEMY_MTL, STARFIGHTER_GLITCH_CYAN_MTL, STARFIGHTER_GLITCH_WHITE_MTL,
        STARFIGHTER_MODEL_OBJ, STARFIGHTER_PLAYER_MTL,
    },
};

pub fn load_and_register_world_models(
    gpu_context: &GpuContext,
    asset_server: &mut AssetServer,
    _world: &mut World,
) {
    // Player
    load_obj(
        "starfighter",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_PLAYER_MTL,
        &gpu_context,
        None,
        1,
        asset_server,
    );

    // Enemy
    load_obj(
        "starfighter_enemy",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_ENEMY_MTL,
        &gpu_context,
        None,
        50,
        asset_server,
    );
    // Cube
    load_obj(
        "cube",
        CUBE_PREFAB_OBJ,
        CUBE_PREFAB_MTL,
        &gpu_context,
        None,
        64,
        asset_server,
    );

    // Laser
    load_obj(
        "laser",
        LASER_MODEL_OBJ,
        LASER_MODEL_MTL,
        gpu_context,
        None,
        1024,
        asset_server,
    );

    // Explosion
    load_obj(
        "explosion",
        EXPLOSION_MODEL_OBJ,
        EXPLOSION_MODEL_MTL,
        gpu_context,
        None,
        1024,
        asset_server,
    );

    // Pickup
    load_obj(
        "pickup",
        PICKUP_MODEL_OBJ,
        PICKUP_MODEL_MTL,
        gpu_context,
        None,
        32,
        asset_server,
    );
    // Pickup variants used while Hyperdrive is active: the base swaps to a
    // pure-white version and a cyan glitch sibling is spawned alongside.
    load_obj(
        "pickup_white",
        PICKUP_MODEL_OBJ,
        PICKUP_WHITE_MTL,
        gpu_context,
        None,
        32,
        asset_server,
    );
    load_obj(
        "pickup_glitch_cyan",
        PICKUP_MODEL_OBJ,
        PICKUP_GLITCH_CYAN_MTL,
        gpu_context,
        None,
        32,
        asset_server,
    );

    // Shield
    load_obj(
        "shield",
        SHIELD_MODEL_OBJ,
        SHIELD_MODEL_MTL,
        gpu_context,
        None,
        2,
        asset_server,
    );

    // Glitch variants — same OBJ as the base model, recoloured + alpha'd
    // material. Two children spawn per "glitch parent" (one cyan, one
    // magenta) and their local Transform.position is shoogled by vfx_system
    // for the chromatic-aberration look.
    load_obj(
        "starfighter_glitch_cyan",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_GLITCH_CYAN_MTL,
        gpu_context,
        None,
        2,
        asset_server,
    );
    load_obj(
        "starfighter_glitch_white",
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_GLITCH_WHITE_MTL,
        gpu_context,
        None,
        2,
        asset_server,
    );
    load_obj(
        "laser_glitch_cyan",
        LASER_MODEL_OBJ,
        LASER_GLITCH_CYAN_MTL,
        gpu_context,
        None,
        1024,
        asset_server,
    );
    load_obj(
        "laser_glitch_white",
        LASER_MODEL_OBJ,
        LASER_GLITCH_WHITE_MTL,
        gpu_context,
        None,
        1024,
        asset_server,
    );
}
