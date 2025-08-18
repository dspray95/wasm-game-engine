#[derive(Debug)]
pub struct Material {
    pub diffuse_color: [u32; 3],
    pub alpha: f32,
}

impl Material {
    pub fn new(color: [u32; 3], alpha: f32) -> Self {
        Material {
            diffuse_color: color,
            alpha,
        }
    }
}
