use cgmath::Vector2;
use noise::{ NoiseFn, Perlin };
use rand::Rng;

use crate::game::procedural_generation;

pub struct Terrain {
    pub n_vertices: u32,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub n_canyon_vertices: u32,
    pub canyon_vertices: Vec<[f32; 3]>,
    pub canyon_triangles: Vec<u32>,
    _seed: u32,
}

impl Terrain {
    pub fn new(width: u32, length: u32) -> Self {
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

        let mut rng = rand::rng();
        let seed: u32 = rng.random();
        let perlin: Perlin = Perlin::new(seed);

        let y_offset: f32 = 0.0;

        let path_left_edge = width / 2 - 2;
        let path_right_edge = width / 2 + 1;

        procedural_generation::generate_terrain_chunk(
            length,
            width,
            y_offset,
            path_left_edge,
            path_right_edge,
            perlin,
            &mut terrain_vertices,
            &mut terrain_triangles,
            &mut rng,
            Vector2 { x: 0, y: 0 } // chunk_z_offset
        );

        Terrain::generate_canyon(
            length,
            y_offset,
            path_left_edge,
            path_right_edge,
            &mut canyon_vertices,
            &mut canyon_triangles
        );

        Self {
            n_vertices,
            vertices: terrain_vertices,
            triangles: terrain_triangles,
            _seed: seed,
            n_canyon_vertices,
            canyon_vertices,
            canyon_triangles,
        }
    }

    fn generate_terrain(
        length: u32,
        width: u32,
        y_offset: f32,
        path_left_edge: u32,
        path_right_edge: u32,
        perlin: Perlin,
        noise_scale: f64,
        height_multiplier: f32,
        vertices: &mut Vec<[f32; 3]>,
        triangles: &mut Vec<u32>
    ) {
        for z in 0..length {
            for x in 0..width {
                let vertex_y_value = if x >= path_left_edge && x <= path_right_edge {
                    y_offset
                } else {
                    let noise_x = (x as f64) * noise_scale;
                    let noise_z = (z as f64) * noise_scale;
                    (perlin.get([noise_x, noise_z]) as f32) + y_offset * height_multiplier
                };

                vertices.push([x as f32, vertex_y_value, z as f32]);

                if x < width - 1 && z < length - 1 {
                    let current_index = x + z * width;
                    let next_x = x + 1;

                    // Skip triangles if any vertex is in the canyon path
                    let is_in_canyon =
                        (x >= path_left_edge + 1 && x <= path_right_edge - 1) ||
                        (next_x >= path_left_edge + 1 && next_x <= path_right_edge - 1);

                    if !is_in_canyon {
                        let a = current_index;
                        let b = current_index + 1;
                        let c = current_index + width;
                        let d = current_index + width + 1;

                        triangles.extend_from_slice(&[c, d, a]);
                        triangles.extend_from_slice(&[b, a, d]);
                    }
                }
            }
        }
    }

    fn generate_canyon(
        length: u32,
        y_offset: f32,
        path_left_edge: u32,
        path_right_edge: u32,
        canyon_vertices: &mut Vec<[f32; 3]>,
        canyon_triangles: &mut Vec<u32>
    ) {
        let canyon_width = path_right_edge - path_left_edge + 1;
        let canyon_depth = y_offset;

        for z in 0..length {
            for x in path_left_edge..=path_right_edge {
                let vertex_y_value = canyon_depth;
                canyon_vertices.push([x as f32, vertex_y_value, z as f32]);
            }
        }

        for z in 0..length - 1 {
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
