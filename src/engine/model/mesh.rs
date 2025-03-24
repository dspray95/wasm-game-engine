use std::ops::{ Add, Sub };

use cgmath::{ vec3, InnerSpace, Vector3 };
use wgpu::util::DeviceExt;

use crate::engine::instance::Instance;

pub(crate) struct Mesh {
    pub label: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub instances: Vec<Instance>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub index_bind_group: wgpu::BindGroup,
    pub positions_bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn new(
        label: String,
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        num_elements: u32,
        instances: Option<Vec<Instance>>,
        device: &wgpu::Device,
        index_bind_group_layout: &wgpu::BindGroupLayout,
        positions_bind_group_layout: &wgpu::BindGroupLayout
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

        let index_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                layout: index_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: index_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}__index_bind_group", label)),
            })
        );

        let positions_bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                layout: positions_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: vertex_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}__index_bind_group", label)),
            })
        );

        Mesh {
            label,
            vertex_buffer,
            index_buffer,
            num_elements,
            instances,
            instance_buffer,
            index_bind_group,
            positions_bind_group,
        }
    }

    pub(crate) fn add_instance(
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

    pub(crate) fn update_instances(&mut self, device: &wgpu::Device) {
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.instance_buffer = Some(
            //TODO Enginestate -> Queue -> Write buffer for updating instance positions
            device.create_buffer_init(
                &(wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}__instance_buffer", self.label)),
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
