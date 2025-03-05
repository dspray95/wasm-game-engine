use std::sync::Arc;
use cgmath::Rotation3;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{ Window, WindowId };

use crate::game::cube::{ self };
use crate::game::terrain::Terrain;

use super::model::model::{ DrawModel, Model };
use super::resources;
use super::state::EngineState;
use super::texture::Texture;

pub struct App {
    instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    window: Option<Arc<Window>>,
    cube_model: Option<Model>,
    terrain_model: Option<Model>,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self {
            instance,
            engine_state: None,
            window: None,
            cube_model: None,
            terrain_model: None,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self.instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let engine_state = EngineState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width
        ).await;

        let obj_model = resources
            ::load_model_from_file("cube.obj", &engine_state.device).await
            .unwrap();

        let array_model = resources::load_model_from_arrays(
            "array cube",
            cube::VERTICES.to_vec(),
            vec![],
            cube::TRIANGLES.to_vec(),
            &engine_state.device
        );

        let terrain_object = Terrain::new(5, 5);
        let terrain_model = resources::load_model_from_arrays(
            "terrain",
            terrain_object.vertices,
            vec![],
            terrain_object.triangles,
            &engine_state.device
        );
        self.window.get_or_insert(window);
        self.engine_state.get_or_insert(engine_state);
        self.cube_model.get_or_insert(array_model);
        self.terrain_model.get_or_insert(terrain_model);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        let engine_state = self.engine_state.as_mut().unwrap();
        engine_state.resize_surface(width, height);
        engine_state.depth_texture = Texture::create_depth_texture(
            &engine_state.device,
            &engine_state.surface_config,
            "depth_texture"
        );
    }

    fn handle_redraw(&mut self) {
        let engine_state = self.engine_state.as_mut().unwrap();
        let array_model = self.cube_model.as_mut().unwrap();
        let terrain_model = self.terrain_model.as_mut().unwrap();

        // Mesh Rendering //

        let surface_texture = engine_state.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = engine_state.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor { label: None })
        );

        let _window = self.window.as_ref().unwrap();
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
                        view: &engine_state.depth_texture.view,
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
            render_pass.set_bind_group(0, &engine_state.camera.render_pass_data.bind_group, &[]);
            render_pass.set_pipeline(&engine_state.render_pipeline);
            for mesh in &terrain_model.meshes {
                match &mesh.instance_buffer {
                    Some(instance_buffer) => {
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw_model_instanced(
                            &terrain_model,
                            0..mesh.instances.len() as u32,
                            &engine_state.camera.render_pass_data.bind_group,
                            &engine_state.light_bind_group
                        );
                    }
                    None => {
                        continue;
                    }
                }
            }
            render_pass.set_pipeline(&engine_state.wireframe_render_pipeline);
            for mesh in &terrain_model.meshes {
                match &mesh.instance_buffer {
                    Some(instance_buffer) => {
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw_model_instanced(
                            &terrain_model,
                            0..mesh.instances.len() as u32,
                            &engine_state.camera.render_pass_data.bind_group,
                            &engine_state.light_bind_group
                        );
                    }
                    None => {
                        continue;
                    }
                }
            }
        }
        engine_state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn handle_camera_update(&mut self) {
        let engine_state = self.engine_state.as_mut().unwrap();

        engine_state.camera_controller.update_camera(&mut engine_state.camera);
        engine_state.camera.update_view_projeciton();
        engine_state.queue.write_buffer(
            &engine_state.camera.render_pass_data.buffer,
            0,
            bytemuck::cast_slice(&[engine_state.camera.render_pass_data.uniform_buffer])
        );
    }

    fn update(&mut self) {
        let engine_state = self.engine_state.as_mut().unwrap();

        // Move our light around to see effect
        let previous_position: cgmath::Vector3<_> = engine_state.light_uniform.position.into();
        engine_state.light_uniform.position = (
            cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0)) *
            previous_position
        ).into();
        engine_state.queue.write_buffer(
            &engine_state.light_buffer,
            0,
            bytemuck::cast_slice(&[engine_state.light_uniform])
        );
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_camera_update();
                self.update();
                self.handle_redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { state, physical_key: PhysicalKey::Code(keycode), .. },
                ..
            } => {
                let engine_state = self.engine_state.as_mut().unwrap();
                engine_state.camera_controller.process_events(keycode, state);
            }
            _ => (),
        }
    }
}
