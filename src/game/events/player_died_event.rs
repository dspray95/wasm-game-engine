use cgmath::Vector3;

pub struct PlayerDiedEvent {
    pub origin: Vector3<f32>,
    pub velocity: Vector3<f32>,
}
