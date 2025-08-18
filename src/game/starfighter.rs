pub struct Starfighter {
    pub mesh_vertices: [[f32; 3]; 10],
    pub triangles: [u32; 12 * 3],
}

impl Starfighter {
    pub fn new() -> Self {
        let vertices: [[f32; 3]; 10] = [
            [0.0, 0.0, 0.4],
            [-0.05, 0.0, 0.3],
            [0.0, -0.02, 0.3],
            [-0.04, -0.02, 0.1],
            [0.0, -0.05, 0.15],
            [0.0, 0.0, 0.05],
            [-0.15, 0.0, 0.0],
            [0.05, 0.0, 0.3],
            [0.15, 0.0, 0.0],
            [0.04, -0.02, 0.1],
        ];

        // Unflatened here for visual separation in-code
        let triangles_unflattened: [[u32; 3]; 12] = [
            [0, 1, 2],
            [2, 1, 3],
            [2, 3, 4],
            [4, 3, 5],
            [1, 6, 3],
            [7, 0, 2],
            [7, 2, 9],
            [2, 4, 9],
            [5, 3, 6],
            [9, 4, 5],
            [8, 9, 5],
            [8, 7, 9],
        ];

        let mut triangles: [u32; 12 * 3] = [0; 12 * 3];
        let mut i = 0;
        for triangle in triangles_unflattened {
            for triangle_index in triangle {
                triangles[i] = triangle_index;
                i = i + 1;
            }
        }

        Self {
            mesh_vertices: vertices,
            triangles: triangles,
        }
    }
}
