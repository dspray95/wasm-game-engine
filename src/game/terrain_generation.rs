use cgmath::{ vec2, Point3, Vector3 };

use crate::{ engine::{ model::model::Model, state::context::GpuContext }, game::terrain::Terrain };

pub struct TerrainGeneration {
    pub terrain_width: u32,
    pub terrain_length: u32,
    pub n_chunks_generated: u32,
    pub next_breakpoint: f32,
}

impl TerrainGeneration {
    pub fn new(terrain_width: u32, terrain_length: u32) -> Self {
        Self {
            terrain_length,
            terrain_width,
            n_chunks_generated: 0,
            next_breakpoint: 0.0,
        }
    }

    pub fn terrain_update(
        &mut self,
        current_z_position: f32,
        gpu_context: &GpuContext
    ) -> Option<Model> {
        if current_z_position >= self.next_breakpoint {
            self.next_breakpoint = self.next_breakpoint + (self.terrain_length as f32);
            self.n_chunks_generated += 1;
            let z_offset = self.terrain_length * self.n_chunks_generated;

            Some(
                Terrain::new(
                    self.terrain_width,
                    self.terrain_length,
                    vec2(0, z_offset as i32),
                    gpu_context
                )
            )
        } else {
            None
        }
    }

    pub fn get_initial_terrain(&mut self, gpu_context: &GpuContext) -> [Model; 3] {
        let terrain: [Model; 3] = std::array::from_fn(|i| {
            let z_offset = self.terrain_length * (i as u32);
            Terrain::new(
                self.terrain_width,
                self.terrain_length,
                vec2(0, z_offset as i32),
                &gpu_context
            )
        });

        self.n_chunks_generated = 2;
        self.next_breakpoint = self.terrain_length as f32;
        return terrain;
    }

    pub fn update() {}
}
