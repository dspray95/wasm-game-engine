use crate::{
    engine::{
        model::{loader::load_model_from_obj_bytes, model::Model},
        state::context::GpuContext,
    },
    game::assets::include::{STARFIGHTER_MODEL_OBJ, STARFIGHTER_PLAYER_MTL},
};

pub fn load_model(gpu_context: &GpuContext) -> Model {
    load_model_from_obj_bytes(
        STARFIGHTER_MODEL_OBJ,
        STARFIGHTER_PLAYER_MTL,
        gpu_context,
        None,
        1,
    )
}
