use cgmath::Vector3;

use crate::engine::ecs::entity::Entity;

pub struct EnemySpawnManager {
    pub n_enemies_spawned: usize,
    pub z_gap_between_spanws: f32,
    pub last_z_pos_spawned_at: f32,
    pub canyon_center_x: f32,
    pub enemy_spawn_elevation: f32,
    pub enemy_spawn_scale: Vector3<f32>,
    pub enemy_entities: Vec<Entity>,
}
