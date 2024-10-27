use std::sync::Arc;

use wgpu::{core::instance, Adapter, Device, Instance, Queue};
use winit::window::Window;

use super::surface::SurfaceWrapper;

// Context containing global wgpu resources
pub struct Context {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    queue: Queue,
}

impl Context {
    pub async fn init(surface: &mut SurfaceWrapper) -> Context {
        log::info!("Init wgpu...");
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, surface.get())
            .await
            .expect("No suitable GPU adapters found!");
        let adapter_info = adapter.get_info();
        log::info!(
            "Using adapter: {} ({:?})",
            adapter_info.name,
            adapter_info.backend
        );

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    label: None,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
