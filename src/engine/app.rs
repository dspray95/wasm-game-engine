use std::sync::Arc;
use std::time::Instant;
use cgmath::Rotation3;
use wgpu::core::device;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ ElementState, KeyEvent, WindowEvent };
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{ KeyCode, PhysicalKey };
use winit::window::{ Window, WindowId };

use crate::engine::camera;
use crate::engine::fps_counter::FpsCounter;
use crate::engine::scene::scene_manager::SceneManager;
use crate::engine::state::app_state::AppState;
use crate::engine::state::context::GpuContext;
use crate::engine::state::render_state::RenderState;
use super::state::engine_state::EngineState;
use super::texture::Texture;

pub struct App<'a> {
    app_state: AppState<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            app_state: AppState::new(),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: Window = event_loop.create_window(Window::default_attributes()).unwrap();
        pollster::block_on(self.app_state.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => { self.app_state.handle_redraw_requested() }
            WindowEvent::Resized(new_size) => {
                self.app_state.handle_resized(new_size.width, new_size.height);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { state, physical_key: PhysicalKey::Code(key_code), .. },
                ..
            } => {
                self.app_state.handle_keyboard_input(state, key_code);
            }
            _ => (),
        }
    }
}
