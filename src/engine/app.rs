use std::sync::Arc;
use cgmath::Rotation3;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{ Window, WindowId };

use crate::engine::scene::scene_manager::SceneManager;
use crate::engine::state::render_state::RenderState;
use super::state::engine_state::EngineState;
use super::texture::Texture;

pub struct App {
    instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    window: Option<Arc<Window>>,
    scene_manager: Option<SceneManager>,
    render_state: RenderState,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self {
            instance,
            engine_state: None,
            window: None,
            scene_manager: None,
            render_state: RenderState::new(),
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

        self.scene_manager.get_or_insert(SceneManager::new(engine_state.gpu_context()).await);
        self.window.get_or_insert(window);
        self.engine_state.get_or_insert(engine_state);
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

    fn handle_camera_update(&mut self) {
        let engine_state = self.engine_state.as_mut().unwrap();

        engine_state.camera_controller.update_camera(&mut engine_state.camera);
        engine_state.camera.update_view_projeciton();
        engine_state.camera.update_position();
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
                self.render_state.handle_redraw(
                    self.engine_state.as_ref().unwrap().render_context(),
                    &self.scene_manager.as_ref().unwrap().models
                );

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
