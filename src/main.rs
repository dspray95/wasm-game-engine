use wasm_game_engine::{ engine };
use winit::event_loop::{ ControlFlow, EventLoop };

fn main() {
    pollster::block_on(run());
}

async fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("loading non-wasm...");
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        println!("loading wasm...");
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = engine::app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
