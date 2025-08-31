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

pub struct App {
    app_state: Rc<RefCell<AppState>>,
    #[cfg(target_arch = "wasm32")]
    resize_timer_id: Rc<RefCell<Option<i32>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            app_state: Rc::new(RefCell::new(AppState::new())),
            #[cfg(target_arch = "wasm32")]
            resize_timer_id: Rc::new(RefCell::new(None)),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_arch = "wasm32")]
        {
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
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Ok(mut state) = self.app_state.try_borrow_mut() {
                    state.handle_redraw_requested();
                }
            }
            #[cfg(target_arch = "wasm32")]
            WindowEvent::Resized(_) => {
                use wasm_bindgen::{ prelude::Closure, JsCast };
                use web_sys::{ window, HtmlCanvasElement };
                let app_state_clone = Rc::clone(&self.app_state);
                let resize_timer_id_clone = Rc::clone(&self.resize_timer_id);

                // Get the current window. This needs to be done before the closure
                // because the closure runs later, and `self` will be out of scope.
                // We'll pass the canvas dimensions directly to avoid capturing `self`'s window field.
                let canvas = window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("wgpu-canvas"))
                    .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
                    .expect("Canvas not found for resize debounce");

                let current_width = canvas.client_width() as u32;
                let current_height = canvas.client_height() as u32;
                let mut self_timer_id_borrow = self.resize_timer_id.borrow_mut();

                // Clear any previous debounce timer
                if let Some(timer_id) = self_timer_id_borrow.take() {
                    window().unwrap().clear_timeout_with_handle(timer_id);
                    log::debug!("Cleared previous resize timer: {}", timer_id);
                }

                // Set a new debounce timer
                let timeout_ms = 500;
                let closure = Closure::once_into_js({
                    let resize_timer_id_clone_for_closure = Rc::clone(&resize_timer_id_clone);
                    move || {
                        if let Ok(mut state) = app_state_clone.try_borrow_mut() {
                            state.handle_resized(current_width, current_height);
                            // After handling, request a redraw to render with new dimensions
                            if let Some(window_arc) = state.window.as_ref() {
                                window_arc.request_redraw();
                            }
                        } else {
                            log::warn!("Could not process debounced resize - AppState is borrowed");
                        }
                        // Clear the timer ID after it fires
                        *resize_timer_id_clone_for_closure.borrow_mut() = None;
                    }
                });

                let timer_id = window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &closure.as_ref().unchecked_ref(),
                        timeout_ms
                    )
                    .expect("Failed to set resize timeout");

                // Prevent the Closure from being dropped immediately, will be dropped when the timeout fires
                *self_timer_id_borrow = Some(timer_id);
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
                if let Ok(mut app_state) = self.app_state.try_borrow_mut() {
                    app_state.handle_keyboard_input(state, key_code);
                } else {
                    log::warn!("Could not handle keyboard input - app_state is borrowed");
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

    // Determine canvas size
    let canvas = window.canvas().unwrap();
    let width = canvas.client_width() as u32;
    let height = canvas.client_height() as u32;

    let instance = app_state.borrow().instance.clone();
    let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

    let engine_state = crate::engine::state::engine_state::EngineState
        ::new(&instance, surface, &window, width, height).await
        .expect("Failed to create engine state");

    let render_state = crate::engine::state::render_state::RenderState::new(
        engine_state.render_context(),
        &engine_state.surface_config
    );

    let scene_manager = crate::engine::scene::scene_manager::SceneManager::new(
        engine_state.gpu_context()
    ).await;

    if let Ok(mut state) = app_state.try_borrow_mut() {
        state.install_window_state(window.clone(), engine_state, render_state, scene_manager);
        log::info!("AppState GPU initialization complete");

        // First redraw to start render loop
        window.request_redraw();
    }
}
