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
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
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
