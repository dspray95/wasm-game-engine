use std::ops::Range;

use cgmath::Vector3;

use crate::engine::state::context::GpuContext;

use super::mesh::Mesh;

#[derive(Clone, Copy, Debug)]
pub struct ModelBounds {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl ModelBounds {
    pub fn zero() -> Self {
        Self { min: Vector3::new(0.0, 0.0, 0.0), max: Vector3::new(0.0, 0.0, 0.0) }
    }

    pub fn from_vertices(vertices: impl IntoIterator<Item = [f32; 3]>) -> Self {
        let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut any = false;
        for vertex in vertices {
            any = true;
            min.x = min.x.min(vertex[0]);
            min.y = min.y.min(vertex[1]);
            min.z = min.z.min(vertex[2]);
            max.x = max.x.max(vertex[0]);
            max.y = max.y.max(vertex[1]);
            max.z = max.z.max(vertex[2]);
        }
        if any { Self { min, max } } else { Self::zero() }
    }

    pub fn center(&self) -> Vector3<f32> {
        (self.max + self.min) * 0.5
    }

    pub fn half_extents(&self) -> Vector3<f32> {
        (self.max - self.min) * 0.5
    }
}

pub struct Model {
    pub(crate) meshes: Vec<Mesh>,
    pub bounds: ModelBounds,
}

impl Model {
    pub fn scale(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for mesh in self.meshes.iter_mut() {
            mesh.scale(x, y, z, gpu_context);
        }
    }

    pub fn position(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for mesh in self.meshes.iter_mut() {
            mesh.position(x, y, z, gpu_context);
        }
    }

    pub fn rotate(&mut self, rotation: cgmath::Quaternion<f32>, gpu_context: &GpuContext) {
        for mesh in self.meshes.iter_mut() {
            mesh.rotate(rotation, gpu_context);
        }
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32, gpu_context: &GpuContext) {
        for mesh in self.meshes.iter_mut() {
            mesh.translate(x, y, z, gpu_context);
        }
    }

    // Called by render_sync_system — applies the same instance transforms to all meshes.
    // All meshes in a model share transforms (e.g. fuselage and cockpit move together).
    pub fn update_instances(&mut self, queue: &wgpu::Queue, instances: &[crate::engine::instance::InstanceRaw]) {
        for mesh in self.meshes.iter_mut() {
            mesh.update_instances(queue, instances);
        }
    }
}

pub(in crate::engine) trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    );

    #[allow(dead_code)] // May need this later
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup
    );

    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    );
}
