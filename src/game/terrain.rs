use std::time::{ SystemTime, UNIX_EPOCH };

use cgmath::Vector2;
use noise::Perlin;
use rand::{ thread_rng, Rng, SeedableRng };

use crate::{
    engine::{ model::{ material::Material, model::Model }, resources, state::context::GpuContext },
    game::procedural_generation,
};

const RAINBOW_ROAD: bool = true; // useful for debugging chunks

const VIBRANT_COLORS: [[u32; 3]; 10] = [
    [255, 50, 50], // Bright Red
    [50, 255, 50], // Bright Green
    [50, 50, 255], // Bright Blue
    [255, 255, 50], // Bright Yellow
    [255, 50, 255], // Bright Magenta/Pink
    [50, 255, 255], // Bright Cyan
    [255, 128, 0], // Bright Orange
    [128, 50, 255], // Bright Purple
    [50, 255, 128], // Bright Lime Green
    [255, 50, 128], // Bright Hot Pink
];

pub struct Terrain {
    pub n_vertices: u32,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub n_canyon_vertices: u32,
    pub canyon_vertices: Vec<[f32; 3]>,
    pub canyon_triangles: Vec<u32>,
}

impl Terrain {
    pub fn new(
        width: u32,
        length: u32,
        chunk_offset: Vector2<i32>,
        gpu_context: &GpuContext
    ) -> Model {
        let n_total_vertices = width * length;

        let canyon_width = width / 2 + 1 - (width / 2 - 1) + 1; // path_right_edge - path_left_edge + 1
        let n_canyon_vertices = canyon_width * length;
        let n_vertices = n_total_vertices - n_canyon_vertices;

        let mut terrain_vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices as usize);
        let mut terrain_triangles: Vec<u32> = Vec::with_capacity(
            ((width - 1) * (length - 1) * 6) as usize
        );
        let mut canyon_vertices: Vec<[f32; 3]> = Vec::with_capacity(n_canyon_vertices as usize);
        let mut canyon_triangles: Vec<u32> = Vec::with_capacity(
            ((width - 1) * (length - 1) * 6) as usize
        );

        let seed = 500;
        let mut rng: rand::prelude::StdRng = rand::rngs::StdRng::seed_from_u64(seed);
        let perlin = Perlin::new(seed as u32);

        let canyon_color = {
            if RAINBOW_ROAD {
                let mut thread_rng = thread_rng();
                VIBRANT_COLORS[thread_rng.gen_range(0..VIBRANT_COLORS.len())]
            } else {
                [236, 95, 255]
            }
        };

        // This drops the canyon into the terrain, roughly to
        // where the lowest bits of terrain should be
        let canyon_y_offset: f32 = -1.0;

        let path_left_edge = width / 2 - 2;
        let path_right_edge = width / 2 + 1;

        procedural_generation::generate_terrain_chunk(
            length,
            width,
            canyon_y_offset,
            path_left_edge,
            path_right_edge,
            perlin,
            &mut terrain_vertices,
            &mut terrain_triangles,
            &mut rng,
            chunk_offset // chunk_y(aka z)_offset
        );

        Terrain::generate_canyon(
            length,
            canyon_y_offset,
            path_left_edge,
            path_right_edge,
            &mut canyon_vertices,
            &mut canyon_triangles,
            chunk_offset
        );

        Model {
            meshes: vec![
                resources::load_mesh_from_arrays(
                    "terrain landscape",
                    terrain_vertices,
                    vec![],
                    terrain_triangles,
                    gpu_context,
                    Material::new([60, 66, 98], 0.5)
                ),
                resources::load_mesh_from_arrays(
                    "terrain canyon floor",
                    canyon_vertices,
                    vec![],
                    canyon_triangles,
                    gpu_context,
                    Material::new(canyon_color, 1.0)
                )
            ],
        }
    }

    fn generate_canyon(
        length: u32,
        y_offset: f32,
        path_left_edge: u32,
        path_right_edge: u32,
        canyon_vertices: &mut Vec<[f32; 3]>,
        canyon_triangles: &mut Vec<u32>,
        chunk_offset: Vector2<i32>
    ) {
        let canyon_width = path_right_edge - path_left_edge + 1;
        let canyon_depth = y_offset;

        for z in 0..=length {
            for x in path_left_edge..=path_right_edge {
                let vertex_y_value = canyon_depth;
                canyon_vertices.push([
                    x as f32,
                    vertex_y_value,
                    (z as f32) + (chunk_offset.y as f32),
                ]);
            }
        }

        for z in 0..length {
            for x in 0..canyon_width - 1 {
                let current_index = x + z * canyon_width;
                let a = current_index;
                let b = current_index + 1;
                let c = current_index + canyon_width;
                let d = current_index + canyon_width + 1;

                canyon_triangles.extend_from_slice(&[c, d, a]);
                canyon_triangles.extend_from_slice(&[b, a, d]);
            }
        }
    }
}
