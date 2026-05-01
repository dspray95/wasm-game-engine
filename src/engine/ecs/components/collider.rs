use cgmath::Vector3;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ColliderShape {
    AABB {
        #[serde(default = "zero_vec3")]
        offset: Vector3<f32>,
        half_extents: Vector3<f32>,
    },
    Sphere {
        #[serde(default = "zero_vec3")]
        offset: Vector3<f32>,
        radius: f32,
    }, //NO_IMPL
}

fn zero_vec3() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, 0.0)
}
