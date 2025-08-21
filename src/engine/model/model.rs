use std::ops::Range;

use crate::engine::state::context::GpuContext;

use super::mesh::Mesh;

pub struct Model {
    pub(crate) meshes: Vec<Mesh>,
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
}

pub(in crate::engine) trait DrawModel<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    );

    #[allow(dead_code)] // May need this later
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup
    );

    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    );
}
