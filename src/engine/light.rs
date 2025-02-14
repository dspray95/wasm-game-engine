#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    // The paddings here are because WGSL shader structs have
    // to be powers of 2.
    // f32 array of length 3 would be 12 bytes, and the padding
    // brings it up to 16 (2^4).
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub __padding: u32,
}
