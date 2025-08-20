use std::sync::Arc;
use std::time::Instant;
use cgmath::Rotation3;
use wgpu::core::device;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ ElementState, KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{ KeyCode, PhysicalKey };
use winit::window::{ Window, WindowId };

use crate::engine::camera;
use crate::engine::fps_counter::FpsCounter;
use crate::engine::scene::scene_manager::SceneManager;
use crate::engine::state::context::GpuContext;
use crate::engine::state::render_state::RenderState;
use super::state::engine_state::EngineState;
use super::texture::Texture;

pub struct App<'a> {
    instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    window: Option<Arc<Window>>,
    scene_manager: Option<SceneManager>,
    render_state: Option<RenderState<'a>>,
    last_frame_time: Instant,
    delta_time: f32,
    fps_counter: FpsCounter,
    show_fps: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        Self {
            instance,
            engine_state: None,
            window: None,
            scene_manager: None,
            render_state: None,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            fps_counter: FpsCounter::new(),
            show_fps: false,
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

        let render_context = engine_state.render_context();
        let render_state = RenderState::new(render_context, &engine_state.surface_config);

        self.scene_manager.get_or_insert(SceneManager::new(engine_state.gpu_context()).await);
        self.window.get_or_insert(window);
        self.engine_state.get_or_insert(engine_state);
        self.render_state.get_or_insert(render_state);
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
        // Update delta_time
        self.fps_counter.update();
        let now = Instant::now();
        // Min delta_time stops big jumps etc
        self.delta_time = now.duration_since(self.last_frame_time).as_secs_f32().min(0.1);
        self.last_frame_time = now;

        // Update scene
        let engine_state = self.engine_state.as_mut().unwrap();
        // We have to pull these out individually rather than using .gpu_context()
        // because the rust compiler is safe and assumes we're immutably borrowing the whole
        // engine_state, and so prevents us trying to mutate the camera
        let device = &engine_state.device;
        let queue = &engine_state.queue;
        let camera = &mut engine_state.camera;

        let gpu_context = GpuContext {
            device,
            queue,
        };

        self.scene_manager.as_mut().unwrap().update(self.delta_time, gpu_context, camera);
    }
}

impl<'a> ApplicationHandler for App<'a> {
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
                self.render_state
                    .as_mut()
                    .unwrap()
                    .handle_redraw(
                        self.engine_state.as_ref().unwrap().render_context(),
                        &self.scene_manager.as_ref().unwrap().models,
                        if self.show_fps {
                            self.fps_counter.get_fps()
                        } else {
                            -1.0
                        }
                    );

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { state, physical_key: PhysicalKey::Code(key_code), .. },
                ..
            } => {
                self.engine_state
                    .as_mut()
                    .unwrap()
                    .camera_controller.process_events(key_code, state);

                self.scene_manager.as_mut().unwrap().player_control_event(key_code, state);
                let is_pressed = state == ElementState::Pressed;
                if state == ElementState::Pressed {
                    match key_code {
                        KeyCode::Home => {
                            self.show_fps = !self.show_fps;
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
}
