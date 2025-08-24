use std::sync::Arc;
use web_time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{ ElementState };
use winit::keyboard::{ KeyCode };
use winit::window::{ Window };

use crate::engine::fps_counter::FpsCounter;
use crate::engine::scene::scene_manager::SceneManager;
use crate::engine::state::context::GpuContext;
use crate::engine::state::engine_state::EngineState;
use crate::engine::state::render_state::RenderState;
use crate::engine::texture::Texture;

const INITIAL_WINDOW_WIDTH: u32 = 1360;
const INITIAL_WINDOW_HEIGHT: u32 = 768;
const MINIMUM_DELTA_TIME: f32 = 0.1;

pub struct AppState {
    pub instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    pub window: Option<Arc<Window>>,
    scene_manager: Option<SceneManager>,
    render_state: Option<RenderState>,
    last_frame_time: Instant,
    delta_time: f32,
    fps_counter: FpsCounter,
    show_fps: bool,
}

impl AppState {
    pub fn new() -> Self {
        log::info!("Creating instance...");
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        log::info!("Returning App...");

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

    // pub async fn build_window_state(
    //     instance: &wgpu::Instance,
    //     window: Window
    // ) -> Result<(Arc<Window>, EngineState, RenderState, SceneManager), anyhow::Error> {
    //     let window = Arc::new(window);

    //     // Request inner size, canvas may not be painted yet so just fall back
    //     let window_size = window
    //         .request_inner_size(PhysicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT))
    //         .unwrap_or_else(|| {
    //             log::warn!("request_inner_size returned None, using default size fallback");
    //             PhysicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
    //         });

    //     log::info!("created window size: {} x {}", window_size.width, window_size.height);
    //     let surface = instance.create_surface(window.clone()).expect("Failed to create surface!");
    //     log::info!("created surface");
    //     let engine_state = EngineState::new(
    //         instance,
    //         surface,
    //         &window,
    //         INITIAL_WINDOW_WIDTH,
    //         INITIAL_WINDOW_HEIGHT
    //     ).await;
    //     log::info!("Created engine state");

    //     let render_context = engine_state.render_context();
    //     let render_state = RenderState::new(render_context, &engine_state.surface_config);

    //     let scene_manager = SceneManager::new(engine_state.gpu_context()).await;

    //     Ok((window, engine_state, render_state, scene_manager))
    // }

    // Sync setter that only mutably borrows for a moment
    pub fn install_window_state(
        &mut self,
        window: Arc<Window>,
        engine_state: EngineState,
        render_state: RenderState,
        scene_manager: SceneManager
    ) {
        self.window = Some(window);
        self.engine_state = Some(engine_state);
        self.render_state = Some(render_state);
        self.scene_manager = Some(scene_manager);
    }

    // pub async fn set_window(&mut self, window: Window) {
    //     let window: Arc<Window> = Arc::new(window);

    //     let window_size = window
    //         .request_inner_size(PhysicalSize::new(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT))
    //         .unwrap();

    //     log::info!("created window size: {} x {}", window_size.width, window_size.height);
    //     let surface = self.instance
    //         .create_surface(window.clone())
    //         .expect("Failed to create surface!");
    //     log::info!("created surface");

    //     let render_context = engine_state.render_context();
    //     let render_state = RenderState::new(render_context, &engine_state.surface_config);
    //     log::info!("Created render state");
    //     self.scene_manager = Some(SceneManager::new(engine_state.gpu_context()).await);
    //     self.window = Some(window);
    //     self.engine_state = Some(engine_state);
    //     self.render_state = Some(render_state);
    // }

    pub fn handle_resized(&mut self, width: u32, height: u32) {
        if let Some(engine_state) = self.engine_state.as_mut() {
            engine_state.resize_surface(width, height);
            engine_state.depth_texture = Texture::create_depth_texture(
                &engine_state.device,
                &engine_state.surface_config,
                "depth_texture"
            );
        } else {
            log::warn!("handle_resized called but engine_state is not ready yet");
        }
    }

    fn handle_camera_update(&mut self) {
        if let Some(engine_state) = self.engine_state.as_mut() {
            engine_state.camera_controller.update_camera(&mut engine_state.camera);
            engine_state.camera.update_view_projeciton();
            engine_state.camera.update_position();
            engine_state.queue.write_buffer(
                &engine_state.camera.render_pass_data.buffer,
                0,
                bytemuck::cast_slice(&[engine_state.camera.render_pass_data.uniform_buffer])
            );
        } else {
            log::warn!("handle_camera_update called but engine state is not ready yet")
        }
    }

    fn update(&mut self) {
        // Update delta_time
        self.fps_counter.update();
        let now = Instant::now();
        // Min delta_time stops big jumps etc
        self.delta_time = now
            .duration_since(self.last_frame_time)
            .as_secs_f32()
            .min(MINIMUM_DELTA_TIME);
        self.last_frame_time = now;

        // Update scene
        if let Some(engine_state) = self.engine_state.as_mut() {
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
        } else {
            log::warn!("update called but engine state is not ready yet")
        }
    }

    pub fn handle_redraw_requested(&mut self) {
        if self.engine_state.is_none() {
            return;
        }

        self.handle_camera_update();
        self.update();

        let engine_state = self.engine_state.as_ref().unwrap();
        let render_state = self.render_state.as_mut().unwrap();
        let scene_manager = self.scene_manager.as_ref().unwrap();

        render_state.handle_redraw(engine_state.render_context(), &scene_manager.models, if
            self.show_fps
        {
            self.fps_counter.get_fps()
        } else {
            -1.0
        });

        // Schedule next frame (browser-friendly)
        self.window.as_ref().unwrap().request_redraw();
    }

    pub fn handle_keyboard_input(&mut self, state: ElementState, key_code: KeyCode) {
        if let Some(engine_state) = self.engine_state.as_mut() {
            engine_state.camera_controller.process_events(key_code, state);

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
        } else {
            log::warn!("handle_keyboard_input called but engine state is not ready yet")
        }
    }
}
