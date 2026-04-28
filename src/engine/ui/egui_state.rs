use wgpu::rwh::WindowsDisplayHandle;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::engine::ecs::query;

pub struct EguiState {
    pub context: egui::Context,
    pub winit_state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
}

impl EguiState {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: &winit::window::Window
    ) -> Self {
        let context = egui::Context::default();

        let winit_state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None
        );

        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            None, // No depth in egui
            1, // Render 1x to resolved surface
            false
        );

        Self {
            context,
            winit_state,
            renderer,
        }
    }

    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent
    ) -> egui_winit::EventResponse {
        self.winit_state.on_window_event(window, event)
    }

    pub fn run(
        &mut self,
        window: &Window,
        build_ui: impl FnMut(&egui::Context)
    ) -> egui::FullOutput {
        let raw_input = self.winit_state.take_egui_input(window);
        let full_output = self.context.run(raw_input, build_ui);

        self.winit_state.handle_platform_output(window, full_output.platform_output.clone());

        full_output
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        window: &Window,
        full_output: egui::FullOutput
    ) {
        let pixels_per_point = self.winit_state.egui_ctx().pixels_per_point();
        let clipped_primitives = self.context.tessellate(full_output.shapes, pixels_per_point);

        let window_size = window.inner_size();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [window_size.width, window_size.height],
            pixels_per_point,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &clipped_primitives,
            &screen_descriptor
        );

        // Render pass
        {
            let render_pass = encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("egui_render_pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: surface_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
            );

            let mut render_pass = render_pass.forget_lifetime();
            self.renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        // Free deleted textures after render pass is dropped
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
