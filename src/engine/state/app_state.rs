use std::sync::Arc;
use web_time::Instant;
use winit::event::{ ElementState };
use winit::keyboard::{ KeyCode };
use winit::window::{ Window };

use crate::engine::camera::camera::Camera;
use crate::engine::ecs::resources::input_state::InputState;
use crate::engine::ecs::system::{ SystemContext, SystemSchedule };
use crate::engine::ecs::world::World;
use crate::engine::fps_counter::FpsCounter;
use crate::engine::model::model_registry::ModelRegistry;
use crate::engine::scene::scene::Scene;
use crate::engine::state::context::GpuContext;
use crate::engine::state::engine_state::EngineState;
use crate::engine::state::render_state::RenderState;
use crate::engine::texture::Texture;

const MINIMUM_DELTA_TIME: f32 = 0.1;

pub struct AppState {
    pub instance: wgpu::Instance,
    engine_state: Option<EngineState>,
    pub window: Option<Arc<Window>>,
    scene: Option<Box<dyn Scene>>,
    render_state: Option<RenderState>,
    last_frame_time: Instant,
    delta_time: f32,
    fps_counter: FpsCounter,
    show_fps: bool,
    world: Option<World>,
    model_registry: Option<ModelRegistry>,
    system_schedule: Option<SystemSchedule>,
}

impl AppState {
    pub fn new() -> Self {
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        Self {
            instance,
            engine_state: None,
            window: None,
            scene: None,
            render_state: None,
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            fps_counter: FpsCounter::new(),
            show_fps: false,
            world: None,
            model_registry: None,
            system_schedule: None,
        }
    }

    pub fn install_window_state(
        &mut self,
        window: Arc<Window>,
        engine_state: EngineState,
        render_state: RenderState,
        scene: Box<dyn Scene>,
        camera: Camera
    ) {
        let mut world = World::new();
        let model_registry = ModelRegistry::new();
        let mut system_schedule = SystemSchedule::new();

        self.window = Some(window);
        self.engine_state = Some(engine_state);
        self.render_state = Some(render_state);

        // ECS setup
        scene.setup_ecs(&mut system_schedule);
        self.model_registry = Some(model_registry);
        self.system_schedule = Some(system_schedule);
        self.scene = Some(scene);

        // World + Resources
        world.add_resource(InputState::default());
        world.add_resource(camera);
        self.world = Some(world);
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

            self.world
                .as_mut()
                .unwrap()
                .get_resource_mut::<Camera>()
                .unwrap()
                .handle_resized(width, height);

            if let Some(render_state) = self.render_state.as_mut() {
                let width = engine_state.surface_config.width;
                let height = engine_state.surface_config.height;
                let format = engine_state.surface_config.format;
                render_state.resize(
                    &engine_state.device,
                    &engine_state.queue,
                    width,
                    height,
                    format
                );
            }
        }
    }

    fn update(&mut self) {
        // Update delta_time
        self.fps_counter.update();
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
            let input = world.get_resource::<InputState>().unwrap().clone();
            let camera = world.get_resource_mut::<Camera>().unwrap();

            let gpu_context = GpuContext {
                device,
                queue,
            };

            // world and engine_state are separate fields — borrow checker allows disjoint borrows.
            self.scene.as_mut().unwrap().update(self.delta_time, gpu_context, camera, &input);

            // ECS
            let device = &engine_state.device;
            let queue = &engine_state.queue;
            let model_registry = self.model_registry.as_mut().unwrap();

            let mut system_context = SystemContext::new(
                self.delta_time,
                device,
                queue,
                model_registry
            );
            self.system_schedule.as_mut().unwrap().run_all(world, &mut system_context);

            // Clear just_pressed / just_released after all readers have run this frame.
            // Must be at the END — keyboard events arrive before RedrawRequested in the
            // winit event loop, so clearing at the start would wipe them before systems see them.
            if let Some(input) = world.get_resource_mut::<InputState>() {
                input.clear_transient();
            }
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

        let engine_state = self.engine_state.as_ref().unwrap();
        let render_state = self.render_state.as_mut().unwrap();
        let scene = self.scene.as_ref().unwrap();
        let ecs_models = self.model_registry.as_ref().unwrap().models();
        let fps: f32 = if self.show_fps { self.fps_counter.get_fps() } else { -1.0 };
        let world = self.world.as_mut().unwrap();

        render_state.handle_redraw(
            engine_state.render_context(
                &world.get_resource::<Camera>().unwrap().render_pass_data.bind_group
            ),
            scene.models(),
            ecs_models,
            fps
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
