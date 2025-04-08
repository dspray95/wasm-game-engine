pub struct Terrain {
    pub n_vertices: u32,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
}

impl Terrain {
    pub fn new(width: u32, length: u32) -> Self {
        let n_vertices = width * length;
        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(n_vertices as usize);
        let mut triangles: Vec<u32> = Vec::with_capacity(((width - 1) * (length - 1) * 6) as usize);
        let y_offset = -10.0;
        // Triangles in shape
        // a o---o b
        //   | \ |
        // c o---o d
        for z in 0..length {
            for x in 0..width {
                vertices.push([x as f32, y_offset, z as f32]);

                if x < width - 1 && z < length - 1 {
                    let current_index = x + z * width;
                    let a = current_index;
                    let b = current_index + 1;
                    let c = current_index + width;
                    let d = current_index + width + 1;

                    triangles.extend_from_slice(&[c, d, a]);
                    triangles.extend_from_slice(&[b, a, d]);
                }
            }
        }

        Self { n_vertices, vertices, triangles }
    }
}
