use std::{ collections::HashSet, ops::{ Add, Sub }, vec };

use cgmath::{ vec3, InnerSpace, Vector3 };
use wgpu::{ core::instance, util::DeviceExt };

use crate::engine::{ instance::Instance, model::material::Material, state::context::GpuContext };

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    // The paddings here are because WGSL shader structs have
    // to be powers of 2.
    // f32 array of length 3 would be 12 bytes, and the padding
    // brings it up to 16 (2^4).
    pub color: [f32; 3],
    pub alpha: f32,
}

pub(crate) struct Mesh {
    pub _label: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub wireframe_index_buffer: wgpu::Buffer,
    pub wireframe_index_count: u32,
    pub num_elements: u32,
    pub instances: Vec<Instance>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub _material: Material,
    pub color_bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn new(
        label: String,
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        wireframe_index_buffer: wgpu::Buffer,
        wireframe_index_count: u32,
        num_elements: u32,
        instances: Option<Vec<Instance>>,
        device: &wgpu::Device,
        material: Material
    ) -> Mesh {
        let instances = instances.unwrap_or(vec![]);

        let instance_buffer: Option<wgpu::Buffer>;
        if instances.len() > 0 {
            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            instance_buffer = Some(
                device.create_buffer_init(
                    &(wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("{}__instance_buffer", label)),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
                )
            );
        } else {
            instance_buffer = None;
        }

        // Flat color rendering setup //
        // Convert from 0-255 sRGB to linear
        let color_uniform = ColorUniform {
            color: [
                ((material.diffuse_color[0] as f32) / 255.0).powf(2.2),
                ((material.diffuse_color[1] as f32) / 255.0).powf(2.2),
                ((material.diffuse_color[2] as f32) / 255.0).powf(2.2),
            ],
            alpha: material.alpha,
        };

        let color_buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Color Buffer"),
                contents: bytemuck::cast_slice(&[color_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        );

        let color_bind_group_layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
        );
        let color_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: None,
                layout: &color_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: color_buffer.as_entire_binding(),
                    },
                ],
            })
        );

        Mesh {
            _label: label,
            vertex_buffer,
            index_buffer,
            wireframe_index_buffer,
            wireframe_index_count,
            num_elements,
            instances,
            instance_buffer,
            _material: material,
            color_bind_group,
        }
    }

    // TODO: When we have an event loop we should batch these transforms and only update the instance buffer
    // once
    pub fn scale(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for instance in &mut self.instances {
            instance.scale = cgmath::vec3(x, y, z);
        }

        self.update_instance_buffer(gpu_context);
    }

    pub fn position(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for instance in &mut self.instances {
            instance.position = cgmath::vec3(x, y, z);
        }

        self.update_instance_buffer(gpu_context);
    }

    pub fn rotate(&mut self, rotation: cgmath::Quaternion<f32>, gpu_context: &GpuContext) {
        for instance in &mut self.instances {
            instance.rotation = rotation;
        }
        self.update_instance_buffer(gpu_context);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for instance in self.instances.iter_mut() {
            instance.position = cgmath::vec3(
                instance.position.x + x,
                instance.position.y + y,
                instance.position.z + z
            );
        }
        self.update_instance_buffer(gpu_context);
    }

    fn update_instance_buffer(&mut self, gpu_context: &GpuContext) {
        let instance_buffer: Option<wgpu::Buffer>;
        if self.instances.len() > 0 {
            let instances = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            instance_buffer = Some(
                gpu_context.device.create_buffer_init(
                    &(wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("{}__instance_buffer", self._label)),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
                )
            );
        } else {
            instance_buffer = None;
        }
        self.instance_buffer = instance_buffer;
    }

    pub(crate) fn _add_instance(
        &mut self,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>
    ) {
        self.instances.push(Instance {
            position,
            rotation,
            scale,
        });
    }

    pub(crate) fn _update_instances(&mut self, device: &wgpu::Device) {
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.instance_buffer = Some(
            //TODO Enginestate -> Queue -> Write buffer for updating instance positions
            device.create_buffer_init(
                &(wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}__instance_buffer", self._label)),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                })
            )
        );
    }

    pub(crate) fn _remove_instance() {
        todo!("NO_IMPL")
    }
}

pub(crate) fn calculate_normals(
    vertices: &Vec<[f32; 3]>,
    triangle_indices: &Vec<u32>
) -> Vec<[f32; 3]> {
    let triangle_count = (triangle_indices.len() / 3) as usize;
    let mut vertex_normals = vec![vec3(0.0, 0.0, 0.0); vertices.len()];

    for i in 0..triangle_count {
        let triangle_index = i * 3;

        // Find out which vertices make our triangle
        let vertex_index_a = triangle_indices[triangle_index] as usize;
        let vertex_index_b = triangle_indices[triangle_index + 1] as usize;
        let vertex_indec_c = triangle_indices[triangle_index + 2] as usize;

        // Calculate the normal for the triangle
        let triangle_normal = calculate_traingle_normals(
            &vertices[vertex_index_a],
            &vertices[vertex_index_b],
            &vertices[vertex_indec_c]
        );

        // Add to already existing normals for each vertex
        vertex_normals[vertex_index_a] = cgmath::Vector3::add(
            vertex_normals[vertex_index_a],
            triangle_normal
        );
        vertex_normals[vertex_index_b] = cgmath::Vector3::add(
            vertex_normals[vertex_index_b],
            triangle_normal
        );
        vertex_normals[vertex_indec_c] = cgmath::Vector3::add(
            vertex_normals[vertex_indec_c],
            triangle_normal
        );
    }

    // Finally iterate through created normals, normalize the vectors, and convert to arrays
    vertex_normals
        .iter()
        .map(|normal| { cgmath::Vector3::normalize(vec3(normal.x, normal.y, normal.z)).into() })
        .collect::<Vec<_>>()
}

fn calculate_traingle_normals(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> Vector3<f32> {
    let ab = cgmath::Vector3::sub(vec3(a[0], a[1], a[2]), vec3(b[0], b[1], b[2]));
    let ac = cgmath::Vector3::sub(vec3(a[0], a[1], a[2]), vec3(c[0], c[1], c[2]));
    cgmath::Vector3::cross(ab, ac)
}

pub fn triangles_to_lines(triangles: &[u32]) -> Vec<u32> {
    let mut edges = HashSet::new();
    let mut lines = vec![];

    for tri in triangles.chunks_exact(3) {
        let i0 = tri[0];
        let i1 = tri[1];
        let i2 = tri[2];

        let edge_pairs = [
            (i0, i1),
            (i1, i2),
            (i2, i0),
        ];

        for &(a, b) in &edge_pairs {
            let edge = if a < b { (a, b) } else { (b, a) };
            if edges.insert(edge) {
                lines.push(edge.0);
                lines.push(edge.1);
            }
        }
    }

    lines
}
