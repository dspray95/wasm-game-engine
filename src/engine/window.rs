use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{self, Window},
};

use super::context::Context;

pub struct WindowWrapper {
    pub window: Option<Arc<Window>>,
    pub context: Context,
}

impl WindowWrapper {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            window: None, // Window not created until application resumed, since it needs to be created in an active event loop
        }
    }
}

impl ApplicationHandler for WindowWrapper {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Title");

        self.window = Some(Arc::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));
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
