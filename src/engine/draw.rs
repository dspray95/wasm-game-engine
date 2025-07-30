use std::ops::Range;

use super::model::{ mesh::Mesh, model::{ DrawModel, Model } };

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> where 'b: 'a {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        if use_line_index_buffer {
            self.set_index_buffer(mesh.wireframe_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        } else {
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }

        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.set_bind_group(2, color_bind_group, &[]);
        if use_line_index_buffer {
            self.draw_indexed(0..mesh.wireframe_index_count, 0, instances);
        } else {
            self.draw_indexed(0..mesh.num_elements, 0, instances);
        }
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup
    ) {
        self.draw_model_instanced(
            model,
            0..1,
            camera_bind_group,
            light_bind_group,
            color_bind_group,
            false
        );
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        color_bind_group: &'a wgpu::BindGroup,
        use_line_index_buffer: bool
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
                color_bind_group,
                use_line_index_buffer
            );
        }
    }
}
