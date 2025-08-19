use cgmath::vec2;

use crate::game::terrain::Terrain;

pub struct TerrainGeneration {
    pub terrain_width: u32,
    pub terrain_length: u32,
    pub n_chunks_generated: u32,
}

impl TerrainGeneration {
    pub fn new(terrain_width: u32, terrain_length: u32) -> Self {
        Self {
            terrain_length,
            terrain_width,
            n_chunks_generated: 0,
        }
    }

    pub fn get_initial_terrain(&self) -> [Terrain; 3] {
        [
            Terrain::new(self.terrain_width, self.terrain_length, vec2(0, 0)),
            Terrain::new(
                self.terrain_width,
                self.terrain_length,
                vec2(0, (self.terrain_length - 1) as i32)
            ),
            Terrain::new(
                self.terrain_width,
                self.terrain_length,
                vec2(0, (self.terrain_length - 1 * 2) as i32)
            ),
        ]
    }

    pub fn update() {}
}
