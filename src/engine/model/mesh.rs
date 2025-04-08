use std::{ collections::HashSet, ops::{ Add, Sub } };

use cgmath::{ vec3, InnerSpace, Vector3 };
use wgpu::{ core::instance, util::DeviceExt, Buffer, Device };

use crate::engine::{ instance::Instance, state::EngineState };

pub(crate) struct Mesh {
    pub label: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub wireframe_index_buffer: wgpu::Buffer,
    pub wireframe_index_count: u32,
    pub num_elements: u32,
    pub instances: Vec<Instance>,
    pub instance_buffer: Option<wgpu::Buffer>,
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
        device: &wgpu::Device
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
        Mesh {
            label,
            vertex_buffer,
            index_buffer,
            wireframe_index_buffer,
            wireframe_index_count,
            num_elements,
            instances,
            instance_buffer,
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
