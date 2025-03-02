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
    0, 4, 1, 1, 4, 5, 0, 2, 4, 4, 2, 6, 2, 3, 7, 2, 7, 6, 3, 1, 7, 7, 1, 5, 1, 2, 0, 3, 2, 1, 5, 7,
    6, 5, 6, 4,
];

pub const POSITION: [f32; 3] = [5.0, 5.0, 5.0];
