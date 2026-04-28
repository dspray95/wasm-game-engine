use cgmath::Vector3;

use crate::engine::ecs::entity::Entity;

pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
    pub normal: Vector3<f32>, // Vec from A -> B normalised
    pub depth: f32, // How deep the collision crosses the normal
}
