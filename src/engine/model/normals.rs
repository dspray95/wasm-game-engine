use super::vertex::ModelVertex;

pub fn calculate_normal_for_triangle(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    // Generates normals for all triangles in a mesh, perpendicular to the surface of the triangle.

    // let bSubA = new;

    // Point(this.B.x - this.A.x, this.B.y - this.A.y, this.B.z - this.A.z);
    // let cSubA = new;
    // Point(this.C.x - this.A.x, this.C.y - this.A.y, this.C.z - this.A.z);

    // let normalX = bSubA.y * cSubA.z - bSubA.z * cSubA.y;
    // let normalY = bSubA.z * cSubA.x - bSubA.x * cSubA.z;
    // let normalZ = bSubA.x * cSubA.y - bSubA.y * cSubA.x;
    // this.normal = new;
    // Vector(normalX, normalY, normalZ);
    // this.normal = this.normal.unitLengthVector();
    return [1.0, 1.0, 1.0];
}
