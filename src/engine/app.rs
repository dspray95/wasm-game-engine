use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ ElementState, KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{ KeyCode, PhysicalKey };
use winit::window::{ Window, WindowId };

use crate::engine::fps_counter::FpsCounter;
use crate::engine::scene::scene_manager::SceneManager;
use crate::engine::state::context::GpuContext;
use crate::engine::state::render_state::RenderState;
use super::state::engine_state::EngineState;
use super::texture::Texture;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct App<'a> {
    // #[cfg(target_arch = "wasm32")]
    // proxy: Option<winit::event_loop::EventLoopProxy<State>>,
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
            // #[cfg(target_arch = "wasm32")]
            // proxy: Some(event_loop.create_proxy()),
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
        // #[cfg(target_arch = "wasm32")]
        // {
        //     use wasm_bindgen::JsCast;
        //     use winit::platform::web::WindowAttributesExtWebSys;

        //     const CANVAS_ID: &str = "canvas";

        //     let window = wgpu::web_sys::window().unwrap_throw();
        //     let document = window.document().unwrap_throw();
        //     let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
        //     let html_canvas_element = canvas.unchecked_into();
        //     window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        // }

        let window: Window = event_loop.create_window(Window::default_attributes()).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            pollster::block_on(self.set_window(window));
        }

        // #[cfg(target_arch = "wasm32")]
        // {
        //     // Run the future asynchronously and use the
        //     // proxy to send the results to the event loop
        //     if let Some(proxy) = self.proxy.take() {
        //         wasm_bindgen_futures::spawn_local(async move {
        //             assert!(
        //                 proxy
        //                     .send_event(
        //                         State::new(window).await.expect("Unable to create canvas!!!")
        //                     )
        //                     .is_ok()
        //             )
        //         });
        //     }
        // }
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

    // #[allow(unused_mut)]
    // fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
    //     // This is where proxy.send_event() ends up
    //     #[cfg(target_arch = "wasm32")]
    //     {
    //         event.window.request_redraw();
    //         event.resize(event.window.inner_size().width, event.window.inner_size().height);
    //     }
    //     self.state = Some(event);
    // }
}
