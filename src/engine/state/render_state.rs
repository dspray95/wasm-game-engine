use wgpu::{ SurfaceConfiguration, TextureFormat };
use wgpu_text::glyph_brush::ab_glyph::FontRef;
use wgpu_text::glyph_brush::{ Section, Text };
use wgpu_text::{ BrushBuilder, TextBrush };

use crate::engine::state::context::RenderContext;
use crate::engine::{ model::model::Model };
use crate::engine::model::model::DrawModel;

pub struct RenderState<'a> {
    clear_color: wgpu::Color,
    text_brush: TextBrush<FontRef<'a>>,
}

impl<'a> RenderState<'a> {
    pub fn new(render_context: RenderContext, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let font_data = include_bytes!("../../../res/DS-DIGI.TTF");
        println!("Font file size: {} bytes", font_data.len());

        match FontRef::try_from_slice(font_data) {
            Ok(_) => println!("Font loaded successfully"),
            Err(e) => println!("Font loading error: {:?}", e),
        }
        RenderState {
            clear_color: wgpu::Color {
                r: 0.011,
                g: 0.014,
                b: 0.03,
                a: 1.0,
            },
            text_brush: BrushBuilder::using_font_bytes(font_data)
                .unwrap()
                .build(
                    render_context.device,
                    surface_config.width,
                    surface_config.height,
                    surface_config.format
                ),
        }
    }

    pub fn handle_redraw(&mut self, render_context: RenderContext, models: &[Model], fps: f32) {
        //0.118, 0.129, 0.192

        // Mesh Rendering //
        let surface_texture = render_context.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = render_context.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") })
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
                                load: wgpu::LoadOp::Clear(self.clear_color),
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
            // 1. Render wireframes that should be behind transparent objects
            render_pass.set_pipeline(render_context.wireframe_render_pipeline);
            for model in models {
                for mesh in &model.meshes {
                    // Only render if this mesh is transparent (has alpha < 1.0)
                    if mesh._material.alpha < 1.0 {
                        match &mesh.instance_buffer {
                            Some(instance_buffer) => {
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass.draw_mesh_instanced(
                                    mesh,
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

            // 2. Transparent terrain
            render_pass.set_pipeline(render_context.render_pipeline);
            for model in models {
                for mesh in &model.meshes {
                    match &mesh.instance_buffer {
                        Some(instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_mesh_instanced(
                                // Changed this line
                                mesh, // Pass mesh instead of model
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

            // 3. Render wireframes on top
            render_pass.set_pipeline(render_context.wireframe_render_pipeline);
            for model in models {
                for mesh in &model.meshes {
                    if mesh._material.alpha < 1.0 {
                        match &mesh.instance_buffer {
                            Some(instance_buffer) => {
                                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                                render_pass.draw_mesh_instanced(
                                    mesh,
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
            }
        }
        // Some setup for text rendering
        match
            self.text_brush.queue(render_context.device, render_context.queue, [
                Section::default()
                    .add_text(
                        Text::new(&((fps.ceil() as u32).to_string() + " fps"))
                            .with_scale(18.0)
                            .with_color([0.0, 1.0, 0.0, 1.0])
                    )
                    .with_screen_position((10.0, 10.0)),
            ])
        {
            Ok(_) => (),
            Err(err) => {
                panic!("{err}");
            }
        }

        // Text render pass
        if fps != -1.0 {
            {
                let mut text_render_pass = encoder.begin_render_pass(
                    &(wgpu::RenderPassDescriptor {
                        label: Some("Text Render Pass"),
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: &surface_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load, // Don't clear - keep the 3D scene
                                    store: wgpu::StoreOp::Store,
                                },
                            }),
                        ],
                        depth_stencil_attachment: None, // No depth buffer for text
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    })
                );

                // Draw text
                self.text_brush.draw(&mut text_render_pass);
            } // text_render_pass is dropped here
        }

        render_context.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
