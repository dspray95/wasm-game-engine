use std::sync::Arc;

use wgpu::{Surface, SurfaceConfiguration};
use winit::{
    dpi::PhysicalSize,
    event::{Event, StartCause},
    window::Window,
};

use super::context::Context;

// Manages the surface and surface configuration
pub struct SurfaceWrapper {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl SurfaceWrapper {
    pub fn new(surface: Surface<'static>, config: SurfaceConfiguration) -> Self {
        Self {
            surface: surface,
            config: config,
        }
    }

    fn start_condition(e: &Event<()>) -> bool {
        match e {
            Event::NewEvents(StartCause::Init) => true,
            _ => false,
        }
    }

    // fn resume(&mut self, context: &Context, window: Arc<Window>, srgb: bool) {
    //     let window_size = window.inner_size();

    //     log::info!("Surface resumed {window_size:?}");
    //     self.surface = Some(context.instance.create_surface(window).unwrap());

    //     let surface = self.surface.as_ref().unwrap();

    //     //Get default config
    //     let mut config = surface
    //         .get_default_config(&context.adapter, window_size.width, window_size.height)
    //         .expect("Surface not supported by adapter");

    //     let view_format = config.format.add_srgb_suffix();
    //     config.view_formats.push(view_format);

    //     surface.configure(&context.device, &config);
    //     self.config = Some(config);
    // }

    // fn resize(&mut self, context: &Context, size: PhysicalSize<u32>) {
    //     log::info!("Surface resized to {size:?}");
    //     let config = self.config.as_mut().unwrap();
    //     // Make sure we don'r resize to 0, will crash
    //     config.width = size.width.max(1);
    //     config.height = size.height.max(1);

    //     let surface = self.surface.as_ref().unwrap();
    //     surface.configure(&context.device, config);
    // }

    // // Acquire the next surface texture
    // fn acquire(&mut self, context: &Context) -> wgpu::SurfaceTexture {
    //     let surface = self.surface.as_ref().unwrap();
    //     match surface.get_current_texture() {
    //         Ok(frame) => frame,
    //         // Retry on timeout
    //         Err(wgpu::SurfaceError::Timeout) => surface
    //             .get_current_texture()
    //             .expect("Failed to get next surface texture"),
    //         // Reconfigure on outdate or lost surface, or OOM
    //         Err(
    //             wgpu::SurfaceError::Outdated
    //             | wgpu::SurfaceError::Lost
    //             | wgpu::SurfaceError::OutOfMemory,
    //         ) => {
    //             surface.configure(&context.device, self.config());
    //             surface
    //                 .get_current_texture()
    //                 .expect("Failed to get next surface texture")
    //         }
    //     }
    // }

    pub fn get(&self) -> &Surface {
        &self.surface
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
