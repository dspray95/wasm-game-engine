use aoee_rust::engine::{
    context::Context, logger::init_logger, surface::SurfaceWrapper, window::WindowWrapper,
};
use wgpu::Surface;
use winit::{
    application::ApplicationHandler,
    event::{self, *},
    event_loop::{ControlFlow, EventLoop},
    window,
};

async fn start() {
    init_logger();
    log::debug!(
        "Enabled backends: {:?}",
        wgpu::Instance::enabled_backend_features()
    );

    let event_loop = EventLoop::new().unwrap();
    let mut surface = SurfaceWrapper::new();
    let context = Context::init(&mut surface).await;
    let mut window_loop: WindowWrapper = WindowWrapper::new(context, surface);

    log::info!("Entering event loop...");

    let _ = event_loop.run_app(&mut window_loop);
}
fn main() {
    pollster::block_on(start());
}
