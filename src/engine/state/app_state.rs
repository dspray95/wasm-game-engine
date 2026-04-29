use std::sync::Arc;
use web_time::Instant;
use winit::event::{ ElementState };
use winit::keyboard::{ KeyCode };
use winit::window::{ Window };

use crate::engine::assets::server::AssetServer;
use crate::engine::ecs::component_registry::ComponentRegistry;
use crate::engine::ecs::components::camera::camera::{ Camera, SurfaceDimensions };
use crate::engine::ecs::events::collision_event::CollisionEvent;
use crate::engine::ecs::resources::camera::ActiveCamera;
use crate::engine::ecs::world_descriptor::load_world;
use crate::engine::events::event_registry::EventRegistry;
use crate::engine::input::bindings_descriptor::BindingsDescriptor;
use crate::engine::input::input_state::InputState;
use crate::engine::ecs::system::{ SystemContext, SystemSchedule };
use crate::engine::ecs::world::World;
use crate::engine::fps_counter::FpsCounter;
use crate::engine::game_setup::GameSetup;
use crate::engine::state::engine_state::EngineState;
use crate::engine::state::render_state::RenderState;
use crate::engine::texture::Texture;
use crate::engine::state::context::{ EguiContext, GpuContext };
use crate::engine::ui::egui_state::EguiState;
use crate::engine::ui::ui_registry::UIRegistry;
use crate::game;
use crate::game::input::bindings::Bindings;

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

    /// Bootstraps the application: takes the post-window-creation handles
    /// and runs the game-setup pipeline to produce a frame-ready AppState.
    ///
    /// Called once after the winit window has been created (in `App::resumed`),
    /// since most of what's installed here depends on a live GPU surface and
    /// therefore can't exist when `AppState::new` runs.
    ///
    /// Setup pipeline (in order):
    /// 1. Allocate the `World`, `AssetServer`, `SystemSchedule`, and `UIRegistry`.
    /// 2. Install window/engine_state/render_state on `self`.
    /// 3. Run `game_setup.load_assets` to register GPU models with the AssetServer.
    /// 4. Run `game_setup.register_components` to populate the component registry
    ///    with game-specific components (engine components are auto-registered).
    /// 5. Run `game_setup.setup_ecs` and `setup_ui` to register systems and panels.
    /// 6. Add engine-managed resources (input, fps, surface dims, event registry).
    /// 7. Register engine events on the event registry.
    /// 8. Load the bindings RON (if any) so input is usable from this point on.
    /// 9. Load the world's RON file (if any) to spawn declared entities.
    /// 10. Run `game_setup.setup` for game-specific entity/resource setup —
    ///     runs last so it can query/modify entities loaded from the world RON.
    /// 11. Construct egui state.
    /// 12. Move locals into `self`.
    ///
    /// Generic over `G: GameSetup` so the associated `Action` type is known
    /// at the type system level — needed for typed bindings deserialization.
    pub fn bootstrap<G: GameSetup>(
        &mut self,
        window: Arc<Window>,
        engine_state: EngineState,
        render_state: RenderState,
        game_setup: G,
        camera_bind_group_layout: wgpu::BindGroupLayout
    ) {
        let mut world = World::new();
        let mut asset_server = AssetServer::new();
        let mut system_schedule = SystemSchedule::new();
        let mut ui_registry = UIRegistry::new();

        self.window = Some(window);
        self.engine_state = Some(engine_state);
        self.render_state = Some(render_state);

        // Step 3: asset loading
        let render_context = self.engine_state.as_mut().unwrap().render_context(None);
        let gpu_context = GpuContext {
            device: render_context.device,
            queue: render_context.queue,
        };
        game_setup.load_assets(&gpu_context, &mut asset_server, &mut world);

        // Step 4: register game-specific components (engine ones auto-registered)
        let mut component_registry = ComponentRegistry::new();
        game_setup.register_components(&mut component_registry);

        // Step 5: register systems and UI panels
        game_setup.setup_ecs(&mut system_schedule);
        game_setup.setup_ui(&mut ui_registry);

        // Step 6: engine-managed resources
        world.add_resource(InputState::default());
        world.add_resource(FpsCounter::new());
        world.add_resource(camera_bind_group_layout);
        world.add_resource(SurfaceDimensions { width: 1920.0, height: 1080.0 });
        world.add_resource(EventRegistry::new());

        // Step 7: engine events
        world.register_event::<CollisionEvent>();

        // Step 8: bindings — input usable from here on
        if let Some(ron) = game_setup.bindings_ron() {
            if let Ok(descriptor) = ron::from_str::<BindingsDescriptor<G::Action>>(ron) {
                world.add_resource(Bindings::<G::Action>::from_descriptor(descriptor));
            }
        }

        // Step 9: declarative world content from RON
        if let Some(ron) = game_setup.world_ron() {
            if let Err(e) = load_world(ron, &mut world, &component_registry, &asset_server) {
                log::error!("Failed to load world: {:?}", e);
            }
        }

        // Step 10: programmatic game setup. Runs last so it can query/modify
        // entities that were loaded from RON in step 9. Block scope keeps the
        // &mut asset_server borrow short-lived.
        {
            let mut ecs_system_context = SystemContext::new(
                self.delta_time,
                render_context.device,
                render_context.queue,
                &mut asset_server,
            );
            game_setup.setup(&mut world, &mut ecs_system_context);
        }

        // Step 11: egui state
        let engine_state = self.engine_state.as_ref().unwrap();
        let egui_state = EguiState::new(
            &engine_state.device,
            engine_state.surface_config.format,
            self.window.as_ref().unwrap(),
        );

        // Step 12: commit locals to self
        self.world = Some(world);
        self.asset_server = Some(asset_server);
        self.system_schedule = Some(system_schedule);
        self.ui_registry = Some(ui_registry);
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

    /// Update runs once per-frame
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
