use std::{ collections::HashSet, ops::{ Add, Sub }, vec };

use cgmath::{ vec3, InnerSpace, Vector3 };
use wgpu::{ util::DeviceExt };

use crate::engine::{
    instance::{ Instance, InstanceRaw },
    model::{ material::Material, vertex::ModelVertex },
    state::context::GpuContext,
};

/* Holds data for a mesh before its loaded into the gpu buffers */
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub material: Material,
}

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

pub struct Mesh {
    pub label: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub wireframe_index_buffer: wgpu::Buffer,
    pub wireframe_index_count: u32,
    pub num_elements: u32,
    pub instances: Vec<Instance>,
    pub instance_count: u32,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub _material: Material,
    pub color_bind_group: wgpu::BindGroup,
    max_instances: usize,
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
        max_instances: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material: Material
    ) -> Mesh {
        let instances = instances.unwrap_or(vec![]);
        let instance_count = instances.len() as u32;

        debug_assert!(
            instances.len() <= max_instances,
            "Initial instance count {} exceeds max_instances {}",
            instances.len(),
            max_instances
        );

        let instance_buffer = if max_instances > 0 {
            let buffer = device.create_buffer(
                &(wgpu::BufferDescriptor {
                    label: Some(&format!("{}__instance_buffer", label)),
                    size: (max_instances * std::mem::size_of::<InstanceRaw>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            );
            if !instances.is_empty() {
                let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
                queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&instance_data));
            }
            Some(buffer)
        } else {
            None
        };

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
            label,
            vertex_buffer,
            index_buffer,
            wireframe_index_buffer,
            wireframe_index_count,
            num_elements,
            instances,
            instance_count,
            instance_buffer,
            _material: material,
            color_bind_group,
            max_instances,
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
        debug_assert!(
            self.instances.len() <= self.max_instances,
            "Instance count {} exceeds max_instances {} for mesh '{}'",
            self.instances.len(),
            self.max_instances,
            self.label
        );
        if let Some(instance_buffer) = &self.instance_buffer {
            let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            gpu_context.queue.write_buffer(
                instance_buffer,
                0,
                bytemuck::cast_slice(&instance_data)
            );
            self.instance_count = self.instances.len() as u32;
        }
    }

    pub fn create_instance(
        &mut self,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>,
        gpu_context: &GpuContext
    ) {
        self.instances.push(Instance {
            position,
            rotation,
            scale,
        });
        self.update_instance_buffer(gpu_context);
    }

    pub(crate) fn _update_instances(&mut self, device: &wgpu::Device) {
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

    pub fn update_buffers(
        &mut self,
        gpu_context: &GpuContext,
        vertices: &[ModelVertex],
        indices: &[u32]
    ) {
        if
            self.vertex_buffer.size() !=
            ((vertices.len() * std::mem::size_of::<ModelVertex>()) as u64)
        {
            log::warn!(
                "mesh.update_buffers(): New vertex buffer size does not match the current buffer size, not updating buffers"
            );
            return;
        }
        if self.index_buffer.size() != ((indices.len() * std::mem::size_of::<u32>()) as u64) {
            log::warn!(
                "mesh.update_buffers(): New index buffer size does not match the current buffer size, not updating buffers"
            );
            return;
        }

        gpu_context.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        gpu_context.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(indices));

        // Update mesh element counts
        self.num_elements = indices.len() as u32;

        // Recalculate wireframe indices and update that buffer too
        let wireframe_indices = triangles_to_lines(indices);
        gpu_context.queue.write_buffer(
            &self.wireframe_index_buffer,
            0,
            bytemuck::cast_slice(&wireframe_indices)
        );
        self.wireframe_index_count = wireframe_indices.len() as u32;
    }

    // Called by render_sync_system each frame with ECS-driven instance data.
    // Writes into the pre-allocated buffer — no GPU allocation, just a data upload.
    pub fn update_instances(&mut self, queue: &wgpu::Queue, instances: &[InstanceRaw]) {
        debug_assert!(
            instances.len() <= self.max_instances,
            "Instance count {} exceeds max_instances {} for mesh '{}'",
            instances.len(),
            self.max_instances,
            self.label
        );
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
            self.instance_count = instances.len() as u32;
        }
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

/// Converts triangle indices into unique edge pairs for wireframe rendering.
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- triangles_to_lines ---

    #[test]
    fn single_triangle_produces_three_edges() {
        let lines = triangles_to_lines(&[0, 1, 2]);
        assert_eq!(lines.len(), 6, "one triangle = 3 edges = 6 indices");
    }

    #[test]
    fn shared_edge_is_not_duplicated() {
        // Two triangles sharing edge (1,2): [0,1,2] and [2,1,3]
        // Triangle 1 contributes edges: (0,1), (1,2), (0,2)
        // Triangle 2 contributes edges: (1,2) [shared], (1,3), (2,3)
        // Unique edges: (0,1), (1,2), (0,2), (1,3), (2,3) = 5 edges = 10 indices
        let lines = triangles_to_lines(&[0, 1, 2, 2, 1, 3]);
        assert_eq!(lines.len(), 10, "two triangles sharing one edge = 5 unique edges = 10 indices");
    }

    #[test]
    fn empty_input_produces_no_lines() {
        assert!(triangles_to_lines(&[]).is_empty());
    }

    #[test]
    fn all_edges_are_pairs() {
        let lines = triangles_to_lines(&[0, 1, 2, 3, 4, 5]);
        assert_eq!(lines.len() % 2, 0, "output must always have an even number of indices");
    }

    // --- calculate_normals ---

    #[test]
    fn flat_xz_triangle_has_vertical_normal() {
        // Triangle flat on the XZ plane — normal should point along Y axis
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let indices = vec![0, 1, 2];
        let normals = calculate_normals(&vertices, &indices);
        assert_eq!(normals.len(), 3);
        for n in &normals {
            assert!(n[0].abs() < 1e-5, "X component should be ~0");
            assert!(n[2].abs() < 1e-5, "Z component should be ~0");
            assert!(n[1].abs() > 0.99, "Y component should be ~±1");
        }
    }

    #[test]
    fn normals_are_unit_length() {
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.5]];
        let indices = vec![0, 1, 2];
        let normals = calculate_normals(&vertices, &indices);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-5, "normal should be unit length, got {len}");
        }
    }

    #[test]
    fn normal_count_matches_vertex_count() {
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let indices = vec![0, 1, 2, 1, 3, 2];
        let normals = calculate_normals(&vertices, &indices);
        assert_eq!(normals.len(), vertices.len());
    }
}
