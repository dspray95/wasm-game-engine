use cgmath::Vector2;
use noise::{ NoiseFn, Perlin };
use rand::{ rngs::{ StdRng }, Rng };

const NOISE_SCALE: f64 = 1.0;

const NOISE_OCTAVES: f64 = 5.0;
const NOISE_LACUNARITY: f64 = 1.0;
const NOISE_PERSISTENCE: f64 = 1.0;

const MESH_HEIGHT_MULTIPLIER: f32 = 1.0;

pub(crate) fn generate_terrain_chunk(
    length: u32,
    width: u32,
    canyon_y_offset: f32,
    path_left_edge: u32,
    path_right_edge: u32,
    perlin: Perlin,
    vertices: &mut Vec<[f32; 3]>,
    triangles: &mut Vec<u32>,
    rng: &mut StdRng,
    chunk_offset: Vector2<i32>
) {
    let mut noise_amplitude: f64 = 2.0;

    // Setup octave offsets
    let mut octave_offsets: [cgmath::Vector2<i32>; NOISE_OCTAVES as usize] = [
        Vector2 { x: 0, y: 0 };
        NOISE_OCTAVES as usize
    ];
    for octave in 0..NOISE_OCTAVES as u32 {
        let octave_offset_x = rng.random_range(-1000..1000);
        let octave_offset_y = rng.random_range(-1000..1000);
        octave_offsets[octave as usize] = Vector2 {
            x: octave_offset_x,
            y: octave_offset_y,
        };

        noise_amplitude *= NOISE_PERSISTENCE;
    }

    for z in 0..=length {
        for x in 0..width {
            // Sample noise for vertex
            let vertex_y_value = if x >= path_left_edge && x <= path_right_edge {
                // Clamp the height to the canyon path if the vertex is on the edge of the canyon
                canyon_y_offset
            } else {
                // Otherwise sample noise
                //set values for octave
                noise_amplitude = 2.0;
                let mut noise_frequency: f64 = 0.1;
                let mut noise_height = 2.0;

                // Sample noise with octaves
                for octave in 0..NOISE_OCTAVES as u32 {
                    let world_x = (x as f32) + (chunk_offset.x as f32);
                    let world_z = (z as f32) + (chunk_offset.y as f32);

                    let sample_x =
                        (((world_x as f64) + (octave_offsets[octave as usize].x as f64)) /
                            NOISE_SCALE) *
                        noise_frequency;
                    let sample_y =
                        (((world_z as f64) + (octave_offsets[octave as usize].y as f64)) /
                            NOISE_SCALE) *
                        noise_frequency;

                    let sample_value = perlin.get([sample_x, sample_y]);

                    // Update values for next octave
                    noise_height = sample_value * noise_amplitude;
                    noise_amplitude *= NOISE_PERSISTENCE;
                    noise_frequency *= NOISE_LACUNARITY;
                }
                (noise_height as f32) * MESH_HEIGHT_MULTIPLIER
            };

            vertices.push([x as f32, vertex_y_value, (z as f32) + (chunk_offset.y as f32)]);

            // Build triangles
            if x < width - 1 && z < length {
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
