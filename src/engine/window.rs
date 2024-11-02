use std::sync::Arc;

use wgpu::Surface;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{self, Window},
};

use super::{context::Context, surface::SurfaceWrapper};

pub struct WindowWrapper {
    pub window: Option<Arc<Window>>,
    pub context: Context,
    pub surface: Option<SurfaceWrapper>,
}

impl WindowWrapper {
    pub fn new(context: Context, surface: Surface<'static>) -> Self {
        Self {
            context,
            surface: None,
            window: None, // Window & surface not created until application resumed, since it needs to be created in an active event loop
        }
    }
}

impl ApplicationHandler for WindowWrapper {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Title");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let window_size = window.inner_size();

        log::info!("Surface resumed {window_size:?}");
        let surface = self
            .context
            .instance
            .create_surface(window.clone())
            .unwrap();

        //Get default config
        let mut surface_config = surface
            .get_default_config(&self.context.adapter, window_size.width, window_size.height)
            .expect("Surface not supported by adapter");

        let view_format = surface_config.format.add_srgb_suffix();
        surface_config.view_formats.push(view_format);

        surface.configure(&self.context.device, &surface_config);
        self.surface = Some(SurfaceWrapper::new(surface, surface_config));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        if window_id != window.id() {
            ()
        }
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            _ => (),
        }
    }
}
