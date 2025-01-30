use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{ Window, WindowId };

use super::game_state::{ self, GameState };
use super::state::AppState;
use super::texture::Texture;

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
    game_state: GameState,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let game_state = GameState {
            game_objects: vec![],
        };

        Self {
            instance,
            state: None,
            window: None,
            game_state,
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

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width
        ).await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        let state = self.state.as_mut().unwrap();
        state.resize_surface(width, height);
        state.depth_texture = Texture::create_depth_texture(
            &state.device,
            &state.surface_config,
            "depth_texture"
        );
    }

    fn handle_redraw(&mut self) {
        let state = self.state.as_mut().unwrap();

        let surface_texture = state.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = state.device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor { label: None })
        );

        let window = self.window.as_ref().unwrap();
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
                        view: &state.depth_texture.view,
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

            // Render pass setup
            render_pass.set_pipeline(&state.render_pipeline);
            render_pass.set_bind_group(0, &state.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &state.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, state.instance_buffer.slice(..));
            render_pass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw call
            render_pass.draw_indexed(0..state.num_indices, 0, 0..state.instances.len() as _);
        }
        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    fn handle_camera_update(&mut self) {
        let state = self.state.as_mut().unwrap();

        state.camera_controller.update_camera(&mut state.camera);
        state.camera_uniform_buffer.update_view_projeciton(&state.camera);
        state.queue.write_buffer(
            &state.camera_buffer,
            0,
            bytemuck::cast_slice(&[state.camera_uniform_buffer])
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
                self.handle_redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
