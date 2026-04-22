use crate::engine::ecs::entity::Entity;

pub struct ActiveCamera(pub Entity);

pub struct CameraBindGroupLayout(pub wgpu::BindGroupLayout);
