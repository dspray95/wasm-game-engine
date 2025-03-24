

pub struct Terrain {
    pub n_vertices: u32,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
}

impl Terrain {
    pub fn new(width: u32, length: u32) -> Self {
        let n_vertices = width * length;
        let mut vertices: Vec<[f32; 3]> = vec![];
        let mut triangles: Vec<u32> = vec![];
        let y_offset = -10.0;
        // Generate vertices
        for z in 0..length {
            for x in 0..width {
                vertices.push([x as f32, y_offset, z as f32]);
                // Create triangles if we aren't in the final column or row of vertices
                // a o-o b
                //   |/|
                // c o-o d
                if x < width - 1 && z < length - 1 {
                    let current_index = x + z;
                    let index_vertex_a = current_index;
                    // To the right of the current vertex
                    let index_vertex_b = current_index + 1;
                    // Immediately below the current vertex
                    let index_vertex_c = current_index + width;
                    // Below and to the right of the current vertex
                    let index_vertex_d = current_index + width + 1;

                    triangles.extend_from_slice(&[index_vertex_c, index_vertex_d, index_vertex_a]);
                    triangles.extend_from_slice(&[index_vertex_b, index_vertex_a, index_vertex_d]);
                }
            }
        }
        println!("{:?}", vertices);
        println!("{:?}", triangles);
        Self { n_vertices, vertices, triangles }
        // Generate Triangles
    }
}
