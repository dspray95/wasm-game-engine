use wasm_game_engine::engine;
use winit::event_loop::{ ControlFlow, EventLoop };

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = engine::app::App::new();

    event_loop.run_app(&mut app).expect("Failed to run app");
}
