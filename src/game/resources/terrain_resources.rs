pub struct TerrainModelIds(pub [usize; 3]);

pub struct TerrainGeneration {
    pub terrain_width: u32,
    pub terrain_length: u32,
    pub n_chunks_generated: u32,
    pub next_breakpoint: f32,
    pub oldest_terrain_index: u32,
}
