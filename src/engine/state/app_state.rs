use std::sync::Arc;
use web_time::Instant;
use winit::event::{ ElementState };
use winit::keyboard::{ KeyCode };
use winit::window::{ Window };

use crate::engine::assets::server::AssetServer;
use crate::engine::ecs::components::camera::camera::{ Camera, SurfaceDimensions };
use crate::engine::ecs::resources::camera::ActiveCamera;
use crate::engine::events::event_registry::EventRegistry;
use crate::engine::input::input_state::InputState;
use crate::engine::ecs::system::{ SystemContext, SystemSchedule };
use crate::engine::ecs::world::World;
use crate::engine::fps_counter::FpsCounter;
use crate::engine::scene::scene::Scene;
use crate::engine::state::engine_state::EngineState;
use crate::engine::state::render_state::RenderState;
use crate::engine::texture::Texture;
use crate::engine::state::context::EguiContext;
use crate::engine::ui::egui_state::EguiState;
use crate::engine::ui::ui_registry::UIRegistry;

const MINIMUM_DELTA_TIME: f32 = 0.1;

pub struct AppState {
    pub instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    pub window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    last_frame_time: Instant,
    delta_time: f32,
    show_fps: bool,
    world: Option<World>,
    asset_server: Option<AssetServer>,
    system_schedule: Option<SystemSchedule>,
    pub egui_state: Option<EguiState>,
    ui_registry: Option<UIRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        Self {
            instance,
            engine_state: None,
            window: None,
            render_state: None,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            show_fps: false,
            world: None,
            asset_server: None,
            system_schedule: None,
            egui_state: None,
            ui_registry: None,
        }
    }

    pub fn install_window_state(
        &mut self,
        window: Arc<Window>,
        engine_state: EngineState,
        render_state: RenderState,
        scene: Box<dyn Scene>,
        camera_bind_group_layout: wgpu::BindGroupLayout
    ) {
        let mut world = World::new();
        let asset_server = AssetServer::new();
        let mut system_schedule = SystemSchedule::new();
        let mut ui_registry = UIRegistry::new();

        self.window = Some(window);
        self.engine_state = Some(engine_state);
        self.render_state = Some(render_state);

        // ECS + UI setup
        scene.setup_ecs(&mut system_schedule);
        scene.setup_ui(&mut ui_registry);
        self.asset_server = Some(asset_server);
        self.system_schedule = Some(system_schedule);
        self.ui_registry = Some(ui_registry);

        // World + Resources
        world.add_resource(InputState::default());
        world.add_resource(FpsCounter::new());
        world.add_resource(camera_bind_group_layout);
        world.add_resource(SurfaceDimensions {
            width: 1920.0,
            height: 1080.0,
        });
        world.add_resource(EventRegistry::new());

        self.world = Some(world);

        // egui setup
        let engine_state = self.engine_state.as_ref().unwrap();
        let egui_state = EguiState::new(
            &engine_state.device,
            engine_state.surface_config.format,
            self.window.as_ref().unwrap()
        );
        self.egui_state = Some(egui_state);
    }

    pub fn handle_resized(&mut self, width: u32, height: u32) {
        if let Some(engine_state) = self.engine_state.as_mut() {
            if
                engine_state.surface_config.width == width &&
                engine_state.surface_config.height == height
            {
                // Skip resize calls if the dimensions are the same
                return;
            }

            engine_state.resize_surface(width, height);
            engine_state.depth_texture = Texture::create_depth_texture(
                &engine_state.device,
                &engine_state.surface_config,
                "depth_texture"
            );

            // Apply resize to the active camera if one exists
            let world = self.world.as_mut().unwrap();
            if let Some(entity) = world.get_resource::<ActiveCamera>().map(|ac| ac.0) {
                if let Some(camera) = world.get_component_mut::<Camera>(entity) {
                    camera.handle_resized(width, height);
                }
            }

            // Make sure we update the surfcce dimensions resource as well
            if let Some(dims) = world.get_resource_mut::<SurfaceDimensions>() {
                dims.width = width as f32;
                dims.height = height as f32;
            }
        }
    }

    fn update(&mut self) {
        // Update FPS counter (lives in World now)
        if let Some(world) = self.world.as_mut() {
            if let Some(fps_counter) = world.get_resource_mut::<FpsCounter>() {
                fps_counter.update();
            }
        }
        let now = Instant::now();
        // Min delta_time stops big jumps etc
        self.delta_time = now
            .duration_since(self.last_frame_time)
            .as_secs_f32()
            .min(MINIMUM_DELTA_TIME);
        self.last_frame_time = now;

        // Update scene
        if let Some(engine_state) = self.engine_state.as_mut() {
            // We have to pull these out individually rather than using .gpu_context()
            // because the rust compiler is safe and assumes we're immutably borrowing the whole
            // engine_state, and so prevents us trying to mutate the camera
            let device = &engine_state.device;
            let queue = &engine_state.queue;

            let world = self.world.as_mut().unwrap();
            let asset_server = self.asset_server.as_mut().unwrap();

            let mut system_context = SystemContext::new(
                self.delta_time,
                device,
                queue,
                asset_server
            );
            self.system_schedule.as_mut().unwrap().run_all(world, &mut system_context);
        }
    }

    pub fn handle_redraw_requested(&mut self) {
        if self.engine_state.is_none() {
            return;
        }
        if let Some(engine_state) = self.engine_state.as_ref() {
            engine_state.device.poll(wgpu::Maintain::Wait);
        }

        self.update();

        let window = self.window.as_ref().unwrap().clone();
        let egui_state = self.egui_state.as_mut().unwrap();
        let ui_registry = self.ui_registry.as_ref().unwrap();
        let world = self.world.as_mut().unwrap();

        let full_output = egui_state.run(&window, |ctx| {
            ui_registry.draw_all(ctx, world);
        });

        // All input consumers (systems + UI panels) have now run for this frame.
        // Clear just_pressed/just_released so they don't fire again next frame.
        if let Some(input) = world.get_resource_mut::<InputState>() {
            input.clear_transient();
        }

        let engine_state = self.engine_state.as_ref().unwrap();
        let render_state = self.render_state.as_mut().unwrap();
        let ecs_models = self.asset_server.as_ref().unwrap().models();

        let world = self.world.as_ref().unwrap();

        let camera_bind_group = world
            .get_resource::<ActiveCamera>()
            .and_then(|ac| world.get_component::<Camera>(ac.0))
            .map(|camera| &camera.render_pass_data.bind_group);

        render_state.handle_redraw(
            engine_state.render_context(camera_bind_group),
            ecs_models,
            EguiContext { state: egui_state, full_output, window: &window }
        );

        // Schedule next frame (browser-friendly)
        self.window.as_ref().unwrap().request_redraw();
    }

    pub fn handle_keyboard_input(&mut self, state: ElementState, key_code: KeyCode) {
        // AppState is the sole writer of InputState. All game and camera logic reads from it.
        if let Some(world) = self.world.as_mut() {
            if let Some(input) = world.get_resource_mut::<InputState>() {
                input.record(key_code, state);
            }
        }
        // Home key is app-level (FPS overlay toggle) — handled here, not in a system.
        if state == ElementState::Pressed && key_code == KeyCode::Home {
            self.show_fps = !self.show_fps;
        }
    }
}
