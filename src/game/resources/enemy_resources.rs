use cgmath::Vector3;

use crate::{engine::ecs::entity::Entity, game::difficulty::DifficultyCurve};

pub struct EnemySpawnManager {
    pub n_enemies_spawned: usize,
    pub spawn_interval: DifficultyCurve,
    pub time_since_last_spawn: f32,
    pub spawn_horizon_z: f32,
    pub canyon_center_x: f32,
    pub lane_offset: f32,
    pub column_z_offset: f32,
    pub despawn_behind_distance: f32,
    pub enemy_spawn_elevation: f32,
    pub enemy_spawn_scale: Vector3<f32>,
    pub enemy_entities: Vec<Entity>,
}
