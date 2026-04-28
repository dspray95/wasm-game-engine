use cgmath::{ Vector2, vec2 };
use noise::Perlin;
use rand::{ Rng, SeedableRng, rng };

use crate::{
    engine::{
        model::{ material::Material, mesh, model::Model, vertex::ModelVertex },
        resources,
        state::context::GpuContext,
    },
    game::{ helpers::procedural_generation, resources::terrain_resources::TerrainGeneration },
};

const RAINBOW_ROAD: bool = false; // useful for debugging chunks

const VIBRANT_COLORS: [[u32; 3]; 10] = [
    [255, 50, 50],
    [50, 255, 50],
    [50, 50, 255],
    [255, 255, 50],
    [255, 50, 255],
    [50, 255, 255],
    [255, 128, 0],
    [128, 50, 255],
    [50, 255, 128],
    [255, 50, 128],
];

pub struct TerrainMeshData {
    pub terrain_vertices: Vec<[f32; 3]>,
    pub terrain_triangles: Vec<u32>,
    pub canyon_vertices: Vec<[f32; 3]>,
    pub canyon_triangles: Vec<u32>,
}

pub fn get_initial_terrain(
    terrain_generation: &mut TerrainGeneration,
    gpu_context: &GpuContext // We need the gpu context because we're rebuilding the terrain meshes on the fly
) -> [Model; 3] {
    let terrain: [Model; 3] = std::array::from_fn(|i| {
        let z_offset = terrain_generation.terrain_length * (i as u32);
        let mesh_data = generate_mesh_data(
            terrain_generation.terrain_width,
            terrain_generation.terrain_length,
            vec2(0, z_offset as i32)
        );
        create_model_from_data(mesh_data, gpu_context)
    });

    terrain_generation.n_chunks_generated = 2;
    terrain_generation.next_breakpoint = terrain_generation.terrain_length as f32;
    terrain
}

pub fn generate_mesh_data(width: u32, length: u32, chunk_offset: Vector2<i32>) -> TerrainMeshData {
    let mut terrain_vertices: Vec<[f32; 3]> = Vec::new();
    let mut terrain_triangles: Vec<u32> = Vec::new();
    let mut canyon_vertices: Vec<[f32; 3]> = Vec::new();
    let mut canyon_triangles: Vec<u32> = Vec::new();

    let seed = 500;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let perlin = Perlin::new(seed as u32);

    // This drops the canyon into the terrain
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
        chunk_offset
    );

    generate_canyon_mesh(
        length,
        canyon_y_offset,
        path_left_edge,
        path_right_edge,
        &mut canyon_vertices,
        &mut canyon_triangles,
        chunk_offset
    );

    TerrainMeshData { terrain_vertices, terrain_triangles, canyon_vertices, canyon_triangles }
}

fn generate_canyon_mesh(
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
            canyon_vertices.push([x as f32, vertex_y_value, (z as f32) + (chunk_offset.y as f32)]);
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

/// Creates a `Model` object from the generated terrain data.
pub fn create_model_from_data(data: TerrainMeshData, gpu_context: &GpuContext) -> Model {
    let canyon_color = if RAINBOW_ROAD {
        let mut thread_rng = rng();
        VIBRANT_COLORS[thread_rng.random_range(0..VIBRANT_COLORS.len())]
    } else {
        [236, 95, 255]
    };

    Model {
        meshes: vec![
            resources::load_mesh_from_arrays(
                "terrain landscape",
                data.terrain_vertices,
                vec![],
                data.terrain_triangles,
                gpu_context,
                Material::new([60, 66, 98], 0.5),
                None,
                1
            ),
            resources::load_mesh_from_arrays(
                "terrain canyon floor",
                data.canyon_vertices,
                vec![],
                data.canyon_triangles,
                gpu_context,
                Material::new(canyon_color, 1.0),
                None,
                1
            )
        ],
    }
}

/// Generates new terrain mesh data for a chunk based on the player's position.
/// Returns `Some(TerrainMeshData)` when a new chunk needs to be generated.
pub fn terrain_update(
    terrain_generation: &mut TerrainGeneration,
    current_z_position: f32
) -> Option<TerrainMeshData> {
    if current_z_position >= terrain_generation.next_breakpoint {
        terrain_generation.next_breakpoint += terrain_generation.terrain_length as f32;
        terrain_generation.n_chunks_generated += 1;
        let z_offset = terrain_generation.terrain_length * terrain_generation.n_chunks_generated;

        Some(
            generate_mesh_data(
                terrain_generation.terrain_width,
                terrain_generation.terrain_length,
                vec2(0, z_offset as i32)
            )
        )
    } else {
        None
    }
}

pub fn replace_terrain_model_buffers(
    terrain_mesh_data: TerrainMeshData,
    old_terrain_model: &mut Model,
    gpu_context: &GpuContext
) {
    let (terrain_mesh, canyon_mesh) = old_terrain_model.meshes.split_at_mut(1);
    let terrain_mesh = &mut terrain_mesh[0];
    let canyon_mesh = &mut canyon_mesh[0];

    let terrain_normals = mesh::calculate_normals(
        &terrain_mesh_data.terrain_vertices,
        &terrain_mesh_data.terrain_triangles
    );

    let canyon_normals = mesh::calculate_normals(
        &terrain_mesh_data.canyon_vertices,
        &terrain_mesh_data.canyon_triangles
    );

    let terrain_vertices = (0..terrain_mesh_data.terrain_vertices.len())
        .map(|i| {
            ModelVertex {
                position: [
                    terrain_mesh_data.terrain_vertices[i][0],
                    terrain_mesh_data.terrain_vertices[i][1],
                    terrain_mesh_data.terrain_vertices[i][2],
                ],
                tex_coords: [0.0, 0.0],
                normal: terrain_normals[i],
            }
        })
        .collect::<Vec<_>>();

    let canyon_vertices = (0..terrain_mesh_data.canyon_vertices.len())
        .map(|i| {
            ModelVertex {
                position: [
                    terrain_mesh_data.canyon_vertices[i][0],
                    terrain_mesh_data.canyon_vertices[i][1],
                    terrain_mesh_data.canyon_vertices[i][2],
                ],
                tex_coords: [0.0, 0.0],
                normal: canyon_normals[i],
            }
        })
        .collect::<Vec<_>>();

    terrain_mesh.update_buffers(
        gpu_context,
        &terrain_vertices,
        &terrain_mesh_data.terrain_triangles
    );
    canyon_mesh.update_buffers(gpu_context, &canyon_vertices, &terrain_mesh_data.canyon_triangles);
}
