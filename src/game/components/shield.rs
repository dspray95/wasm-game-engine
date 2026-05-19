use cgmath::Vector3;

use crate::engine::ecs::entity::Entity;

pub struct Shield {
    pub time_remaining: f32,
    pub visual_entity: Entity,
    pub rotation_axis: Vector3<f32>,
    pub flash_phase_timer: f32,
}
