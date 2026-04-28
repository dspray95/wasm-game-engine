use wgpu_text::glyph_brush::ab_glyph::{ FontRef, FontVec };
use wgpu_text::glyph_brush::{ Section, Text };
use wgpu_text::{ BrushBuilder, TextBrush };

use crate::engine::state::context::{ EguiContext, RenderContext };
use crate::engine::{ model::model::Model };
use crate::engine::model::model::DrawModel;

pub struct RenderState {
    clear_color: wgpu::Color,
    // text_brush: TextBrush<FontVec>,
}

impl RenderState {
    pub fn new() -> Self {
        let font_data = include_bytes!("../../../res/DS-DIGI.TTF");

        match FontRef::try_from_slice(font_data) {
            Ok(_) => (),
            Err(e) => println!("Font loading error: {:?}", e),
        }

        let font = FontVec::try_from_vec(font_data.into()).expect("Failed to load font");

        RenderState {
            clear_color: wgpu::Color {
                r: 0.081,
                g: 0.084,
                b: 0.14,
                a: 1.0,
            },
            // text_brush: BrushBuilder::using_font(font).build(
            //     render_context.device,
            //     surface_config.width,
            //     surface_config.height,
            //     surface_config.format
            // ),
        }
    }

    pub fn handle_redraw(
        &mut self,
        render_context: RenderContext,
        ecs_models: &[Model],
        egui_context: EguiContext,
        fps: f32
    ) {
        // Mesh Rendering //
        let surface_texture = render_context.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut command_encoder = render_context.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") })
        );

        {
            // Clear pass
            command_encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &render_context.msaa_texture_view,
                            resolve_target: Some(&surface_view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(self.clear_color),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                    ..Default::default()
                })
            );
        }

        // We use a guard scope here because render_pass holds a borrow of
        // the command encoder, so we need to drop it before calling encoder.finish()
        if let Some(camera_bind_group) = render_context.camera_bind_group {
            // Render pass init
            let mut render_pass: wgpu::RenderPass<'_> = command_encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &render_context.msaa_texture_view, // render into MSAA texture
                            resolve_target: Some(&surface_view), // resolve to swap chain
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &render_context.msaa_depth_texture_view,
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
            // 1. Render wireframes that should be behind transparent objects
            render_pass.set_pipeline(render_context.wireframe_render_pipeline);
            for model in ecs_models {
                for mesh in &model.meshes {
                    // Only render if this mesh is transparent (has alpha < 1.0)
                    if mesh._material.alpha < 1.0 {
                        match &mesh.instance_buffer {
                            Some(instance_buffer) => {
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass.draw_mesh_instanced(
                                    mesh,
                                    0..mesh.instance_count,
                                    camera_bind_group,
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

            // 2. Opaque geometry
            render_pass.set_pipeline(render_context.render_pipeline);
            for model in ecs_models {
                for mesh in &model.meshes {
                    match &mesh.instance_buffer {
                        Some(instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_mesh_instanced(
                                // Changed this line
                                mesh, // Pass mesh instead of model
                                0..mesh.instance_count,
                                camera_bind_group,
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
            // 3. Render wireframes on top
            render_pass.set_pipeline(render_context.wireframe_render_pipeline);
            for model in ecs_models {
                for mesh in &model.meshes {
                    if mesh._material.alpha < 1.0 {
                        match &mesh.instance_buffer {
                            Some(instance_buffer) => {
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass.draw_mesh_instanced(
                                    mesh,
                                    0..mesh.instance_count,
                                    camera_bind_group,
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
            }
        }
        // Some setup for text rendering
        // match
        //     self.text_brush.queue(render_context.device, render_context.queue, [
        //         Section::default()
        //             .add_text(
        //                 Text::new(&((fps.ceil() as u32).to_string() + " fps"))
        //                     .with_scale(18.0)
        //                     .with_color([0.0, 1.0, 0.0, 1.0])
        //             )
        //             .with_screen_position((10.0, 10.0)),
        //     ])
        // {
        //     Ok(_) => (),
        //     Err(err) => {
        //         panic!("{err}");
        //     }
        // }

        // egui pass — composites the UI on top of the 3D scene
        egui_context.state.render(
            render_context.device,
            render_context.queue,
            &mut command_encoder,
            &surface_view,
            egui_context.window,
            egui_context.full_output,
        );

        render_context.queue.submit(Some(command_encoder.finish()));
        surface_texture.present();
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat
    ) {
        // We need to make sure the text brush is rebuild on re-size
        // let font_data = include_bytes!("../../../res/DS-DIGI.TTF");
        // let font = FontVec::try_from_vec(font_data.into()).expect("Failed to load font");
        // self.text_brush.resize_view(width as f32, height as f32, queue);
        // self.text_brush = BrushBuilder::using_font(font).build(device, width, height, format);
    }
}
