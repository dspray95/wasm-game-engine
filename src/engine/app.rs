use std::cell::RefCell;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::{ KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{ Window, WindowId };
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use crate::engine::state::app_state::AppState;

const INITIAL_WINDOW_WIDTH: u32 = 1920;
const INITIAL_WINDOW_HEIGHT: u32 = 1080;

pub struct App {
    app_state: Rc<RefCell<AppState>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            app_state: Rc::new(RefCell::new(AppState::new())),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.app_state.borrow().window.is_some() {
                return;
            }

            use std::sync::Arc;
            use crate::engine::state::{ engine_state::EngineState, render_state::RenderState };
            use crate::game::canyon_runner_world::CanyonRunnerWorld;

            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes().with_inner_size(
                            winit::dpi::LogicalSize::new(
                                INITIAL_WINDOW_WIDTH,
                                INITIAL_WINDOW_HEIGHT
                            )
                        )
                    )
                    .expect("Failed to create window")
            );

            let instance = self.app_state.borrow().instance.as_ref().unwrap().clone();
            let surface = instance
                .create_surface(window.clone())
                .expect("Failed to create surface");

            let size = window.inner_size();

            let (engine_state, render_state, _world, camera_bind_group_layout) = pollster::block_on(
                async {
                    let (engine_state, camera_bind_group_layout) = EngineState::new(
                        &instance,
                        surface,
                        &window,
                        size.width,
                        size.height
                    ).await.expect("Failed to create engine state");

                    let render_state = RenderState::new();

                    let world: Box<CanyonRunnerWorld> = Box::new(CanyonRunnerWorld);

                    (engine_state, render_state, world, camera_bind_group_layout)
                }
            );

            // TODO: Make this more generic so App doesn't need to know the specifics
            // of the initial game setup
            let game_setup = CanyonRunnerWorld;

            self.app_state
                .borrow_mut()
                .bootstrap(
                    window.clone(),
                    engine_state,
                    render_state,
                    game_setup,
                    camera_bind_group_layout
                );

            window.request_redraw();
        }

        #[cfg(target_arch = "wasm32")]
        {
            if self.app_state.borrow().window.is_some() {
                // skip reinitialization if the app already exists
                return;
            }

            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            let canvas = web_sys
                ::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("wgpu-canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .expect("Canvas not found");

            let window_attributes = Window::default_attributes().with_canvas(Some(canvas));
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            let app_state = Rc::clone(&self.app_state);

            // Async GPU setup
            wasm_bindgen_futures::spawn_local(async move {
                initialize_gpu_for_wasm(app_state, window).await;
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Forward events to egui
        let egui_consumed_event = if let Ok(mut state) = self.app_state.try_borrow_mut() {
            // Clone the window Arc first so we don't hold an immutable borrow on
            // `state` when we mutably borrow state.egui_state below.
            let window = state.window.as_ref().cloned();
            if let (Some(egui_state), Some(window)) = (state.egui_state.as_mut(), window) {
                let response = egui_state.on_window_event(&window, &event);
                if response.repaint {
                    window.request_redraw();
                }
                response.consumed
            } else {
                false
            }
        } else {
            false
        };

        match event {
            WindowEvent::CloseRequested => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(mut state) = self.app_state.try_borrow_mut() {
                    state.release_gpu_resources();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Ok(mut state) = self.app_state.try_borrow_mut() {
                    state.handle_redraw_requested();
                }
            }
            #[cfg(target_arch = "wasm32")]
            WindowEvent::Resized(new_size) => {
                // On web, wgpu's surface texture size comes from the canvas's
                // backing-store attributes (canvas.width/height), not from what
                // we pass to surface.configure(). Winit doesn't synchronise those
                // with what it reports as inner_size, so we set them explicitly
                // here to match the size we configure the surface with.
                use wasm_bindgen::JsCast;
                if let Some(canvas) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("wgpu-canvas"))
                    .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    canvas.set_width(new_size.width);
                    canvas.set_height(new_size.height);
                }
                if let Ok(mut state) = self.app_state.try_borrow_mut() {
                    state.handle_resized(new_size.width, new_size.height);
                    if let Some(window_arc) = state.window.as_ref() {
                        window_arc.request_redraw();
                    }
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(new_size) => {
                if let Ok(mut state) = self.app_state.try_borrow_mut() {
                    state.handle_resized(new_size.width, new_size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { state, physical_key: PhysicalKey::Code(key_code), .. },
                ..
            } => {
                if !egui_consumed_event {
                    if let Ok(mut app_state) = self.app_state.try_borrow_mut() {
                        app_state.handle_keyboard_input(state, key_code);
                    } else {
                        log::warn!("Could not handle keyboard input");
                    }
                }
            }
            _ => (),
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn initialize_gpu_for_wasm(app_state: Rc<RefCell<AppState>>, window: Window) {
    use std::sync::Arc;
    let window = Arc::new(window);

    let canvas = window.canvas().unwrap();

    // getBoundingClientRect() returns the CSS-rendered size in logical pixels.
    // Multiplying by devicePixelRatio gives physical pixels, which is what wgpu
    // needs for the surface. On macOS/Windows the canvas width/height *attributes*
    // default to 300x150 (browser default for a bare <canvas> element), so reading
    // inner_size() before setting those attributes returns stale values and the
    // surface gets configured at 300x150 — causing a pink screen until the first
    // resize event fires with the correct viewport dimensions.
    let dpr = web_sys::window()
        .map(|w| w.device_pixel_ratio())
        .unwrap_or(1.0);
    let rect = canvas.get_bounding_client_rect();
    let width = ((rect.width() * dpr) as u32).max(1);
    let height = ((rect.height() * dpr) as u32).max(1);

    // Set the backing-store attributes so that winit's inner_size() (which reads
    // canvas.width/canvas.height) agrees with the physical size we pass to wgpu.
    canvas.set_width(width);
    canvas.set_height(height);

    let instance = app_state.borrow().instance.as_ref().unwrap().clone();
    let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

    let (engine_state, camera_bind_group_layout) = crate::engine::state::engine_state::EngineState
        ::new(&instance, surface, &window, width, height).await
        .expect("Failed to create engine state");

    let render_state = crate::engine::state::render_state::RenderState::new();

    let game_setup = crate::game::canyon_runner_world::CanyonRunnerWorld;

    if let Ok(mut state) = app_state.try_borrow_mut() {
        state.bootstrap(
            window.clone(),
            engine_state,
            render_state,
            game_setup,
            camera_bind_group_layout
        );
        window.request_redraw();
    }
}
