use crate::engine::state::context::RenderContext;
use crate::engine::{model::model::Model};
use crate::engine::model::model::DrawModel; 

pub struct RenderState {}

impl RenderState {
    
    pub fn handle_redraw(render_context: RenderContext, models: &[Model]) {
    
        // Mesh Rendering //
        let surface_texture = render_context.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = render_context.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor { label: None })
        );

        // We use a guard scope here because render_pass holds a borrow of
        // the command encoder, so we need to drop it before calling encoder.finish()
        {
            // Render pass init
            let mut render_pass = encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &surface_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: render_context.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                })
            );

            // Render pass
            render_pass.set_bind_group(0, render_context.camera_bind_group, &[]);
            render_pass.set_pipeline(render_context.render_pipeline);
            for model in models {
                for mesh in &model.meshes {
                    match &mesh.instance_buffer {
                        Some(instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_model_instanced(
                                &model,
                                0..mesh.instances.len() as u32,
                                render_context.camera_bind_group,
                                render_context.light_bind_group,
                                &mesh.color_bind_group,
                                false
                            );
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }

            // Wireframe Pass
            render_pass.set_pipeline(render_context.wireframe_render_pipeline);
            for model in models {
                for mesh in &model.meshes {
                    match &mesh.instance_buffer {
                        Some(instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_model_instanced(
                                &model,
                                0..mesh.instances.len() as u32,
                                render_context.camera_bind_group,
                                render_context.light_bind_group,
                                &mesh.color_bind_group,
                                true
                            );
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }
        }
        render_context.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}