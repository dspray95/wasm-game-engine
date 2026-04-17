#[derive(serde::Deserialize)]
pub struct ModelDescriptor {
    pub meshes: Vec<MeshDescriptor>,
}

#[derive(serde::Deserialize)]
pub struct MeshDescriptor {
    pub label: String,
    pub vertices: Vec<(f32, f32, f32)>,
    pub triangles: Vec<u32>,
    pub material: MaterialDescriptor,
    pub max_instances: usize,
}

#[derive(serde::Deserialize)]
pub struct MaterialDescriptor {
    pub diffuse_color: (u32, u32, u32),
    pub alpha: f32,
}
