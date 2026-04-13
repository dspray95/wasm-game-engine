#[derive(Copy, Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    // InstanceRaw is used because there is no Quaternion type
    // object in the shaders
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

// To use instances in wgsl shaders we need to convert them to raw data first, before creating the
// instance buffer like so:
//
//      let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
//      let instance_buffer = device.create_buffer_init(
//         &(wgpu::util::BufferInitDescriptor {
//              label: Some("Instance Buffer"),
//              contents: bytemuck::cast_slice(&instance_data),
//              usage: wgpu::BufferUsages::VERTEX,
//          })
//      );

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (
                cgmath::Matrix4::from_translation(self.position) *
                cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
                cgmath::Matrix4::from(self.rotation)
            ).into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

/// Size of InstanceRaw in bytes — must match the vertex buffer layout offsets in `desc()`.
/// model (4×4 f32 = 64 bytes) + normal (3×3 f32 = 36 bytes) = 100 bytes.
pub const INSTANCE_RAW_SIZE: usize = std::mem::size_of::<InstanceRaw>();

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model //
                // A mat4 (ie 4x4 matrix) takes up 4 vertex slots as it is technically 4 vec4s
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // normals //
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{ vec3, Quaternion, Rotation3, SquareMatrix };

    fn identity_instance() -> Instance {
        Instance {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(0.0)),
            scale: vec3(1.0, 1.0, 1.0),
        }
    }

    #[test]
    fn instance_raw_size_is_100_bytes() {
        // model (4×4 f32 = 64) + normal (3×3 f32 = 36) = 100 bytes.
        // The buffer pre-allocation in Mesh::new() relies on this size being stable.
        // If this test fails, update the vertex attribute offsets in desc() too.
        assert_eq!(INSTANCE_RAW_SIZE, 100);
    }

    #[test]
    fn identity_instance_produces_identity_model_matrix() {
        let raw = identity_instance().to_raw();
        let expected: [[f32; 4]; 4] = cgmath::Matrix4::identity().into();
        for (r, e) in raw.model.iter().zip(expected.iter()) {
            for (a, b) in r.iter().zip(e.iter()) {
                assert!((a - b).abs() < 1e-6, "model matrix mismatch: {a} != {b}");
            }
        }
    }

    #[test]
    fn translation_appears_in_last_column_of_model_matrix() {
        let instance = Instance {
            position: vec3(3.0, 5.0, 7.0),
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(0.0)),
            scale: vec3(1.0, 1.0, 1.0),
        };
        let raw = instance.to_raw();
        // In a column-major matrix, translation is in the last column (index 3): [tx, ty, tz, 1]
        assert!((raw.model[3][0] - 3.0).abs() < 1e-6);
        assert!((raw.model[3][1] - 5.0).abs() < 1e-6);
        assert!((raw.model[3][2] - 7.0).abs() < 1e-6);
    }

    #[test]
    fn uniform_scale_appears_on_matrix_diagonal() {
        let instance = Instance {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(0.0)),
            scale: vec3(2.0, 2.0, 2.0),
        };
        let raw = instance.to_raw();
        assert!((raw.model[0][0] - 2.0).abs() < 1e-6);
        assert!((raw.model[1][1] - 2.0).abs() < 1e-6);
        assert!((raw.model[2][2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn normal_matrix_is_rotation_only() {
        // The normal matrix should not be affected by translation or scale
        let a = Instance {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(45.0)),
            scale: vec3(1.0, 1.0, 1.0),
        };
        let b = Instance {
            position: vec3(99.0, 99.0, 99.0),
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(45.0)),
            scale: vec3(5.0, 5.0, 5.0),
        };
        let raw_a = a.to_raw();
        let raw_b = b.to_raw();
        for (ra, rb) in raw_a.normal.iter().zip(raw_b.normal.iter()) {
            for (a, b) in ra.iter().zip(rb.iter()) {
                assert!(
                    (a - b).abs() < 1e-6,
                    "normal matrices should match regardless of position/scale"
                );
            }
        }
    }
}
