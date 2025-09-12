use anyhow::Ok;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_game_engine::engine;
use winit::event_loop::{ ControlFlow, EventLoop };

fn main() {
    pollster::block_on(run());
}

async fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("loading non-wasm...");
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).expect("Failed to init logger");
        console_error_panic_hook::set_once();
    }

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = engine::app::App::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.run_app(&mut app).expect("Failed to run app");
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
    }

    Ok(())
}
