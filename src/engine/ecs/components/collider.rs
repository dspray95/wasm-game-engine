use cgmath::Vector3;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ColliderShape {
    AABB {
        half_extents: Vector3<f32>,
    },
    Sphere {
        radius: f32,
    }, //NO_IMPL
}
