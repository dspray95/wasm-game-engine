pub mod engine;
pub mod game;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::{ ControlFlow, EventLoop };

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to init logger");
    log::info!("WASM start called");

    wasm_bindgen_futures::spawn_local(run());
}

#[cfg(target_arch = "wasm32")]
async fn run() {
    log::info!("Starting WASM event loop");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait); // align with requestAnimationFrame

    use winit::platform::web::EventLoopExtWebSys;
    let mut app = crate::engine::app::App::new();
    let _ = event_loop.spawn_app(app);
}
