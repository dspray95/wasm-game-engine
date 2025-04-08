pub const SCALE: f32 = 10.0;
const X_OFFSET: f32 = SCALE * 0.5;
const Y_OFFSET: f32 = SCALE * 0.5;
const Z_OFFSET: f32 = SCALE * 0.5;

pub const VERTICES: [[f32; 3]; 8] = [
    [-X_OFFSET, -Y_OFFSET, -Z_OFFSET],
    [X_OFFSET, -Y_OFFSET, -Z_OFFSET],
    [-X_OFFSET, -Y_OFFSET, Z_OFFSET],
    [X_OFFSET, -Y_OFFSET, Z_OFFSET],
    [-X_OFFSET, Y_OFFSET, -Z_OFFSET],
    [X_OFFSET, Y_OFFSET, -Z_OFFSET],
    [-X_OFFSET, Y_OFFSET, Z_OFFSET],
    [X_OFFSET, Y_OFFSET, Z_OFFSET],
];

pub const TRIANGLES: [u32; 36] = [
    // Bottom (-Y)
    0, 1, 2, 2, 1, 3,
    // Top (+Y)
    4, 6, 5, 5, 6, 7,
    // Front (-Z)
    0, 4, 1, 1, 4, 5,
    // Back (+Z)
    2, 3, 6, 6, 3, 7,
    // Left (-X)
    0, 2, 4, 4, 2, 6,
    // Right (+X)
    1, 5, 3, 3, 5, 7,
];

pub const POSITION: [f32; 3] = [5.0, 5.0, 5.0];
