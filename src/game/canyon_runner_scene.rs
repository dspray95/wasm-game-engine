use cgmath::{ vec3, Rotation3 };
use winit::keyboard::KeyCode;

use crate::{
    engine::{
        camera::camera::Camera,
        ecs::{
            components::{ renderable::Renderable, transform::Transform },
            resources::input_state::InputState,
            system::{ SystemContext, SystemSchedule },
            systems::render_sync_system::render_sync_system,
            world::World,
        },
        model::model::Model,
        scene::scene::Scene,
        state::context::GpuContext,
    },
    game::{
        laser::LaserManager,
        starfighter::Starfighter,
        systems::camera_control_system::camera_control_system,
        terrain_generation::TerrainGeneration,
    },
};

const TERRAIN_WIDTH: u32 = 50;
const TERRAIN_LENGTH: u32 = 150;
const MOVEMENT_SPEED: f32 = 10.0;

pub struct CanyonRunnerScene {
    pub models: Vec<Model>, // 0-2 are terrain, 3 is the player, 4 is laser
    pub starfighter: Starfighter,
    pub laser_gun: LaserManager,
    pub terrain_generation: TerrainGeneration,
    pub oldest_terrain_index: u32,
    movement_enabled: bool,
}

impl CanyonRunnerScene {
    pub async fn new(gpu_context: GpuContext<'_>) -> Self {
        // Terrain setup
        let mut terrain_generation = TerrainGeneration::new(TERRAIN_WIDTH, TERRAIN_LENGTH);
        let terrain_models = terrain_generation.get_initial_terrain(&gpu_context);

        // Player model
        let mut starfighter_model = Starfighter::load_model(&gpu_context);
        starfighter_model.position(24.5, -1.0, 3.0, &gpu_context);
        starfighter_model.scale(0.3, 0.3, 0.3, &gpu_context);
        starfighter_model.rotate(
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(180.0)),
            &gpu_context
        );

        // Laser model
        let laser_model = LaserManager::load_model(&gpu_context);

        let mut models: Vec<Model> = terrain_models.into();
        models.push(starfighter_model);
        models.push(laser_model);

        Self {
            models,
            starfighter: Starfighter::new(vec3(24.5, -0.5, 5.0)),
            laser_gun: LaserManager::new(),
            terrain_generation,
            oldest_terrain_index: 0,
            movement_enabled: false,
        }
    }

    fn move_player(
        &mut self,
        delta_time: f32,
        gpu_context: &GpuContext,
        camera: &mut Camera,
        input: &InputState
    ) {
        camera.translate(0.0, 0.0, MOVEMENT_SPEED * delta_time, gpu_context.queue);

        // Move forward
        let starfighter_model = &mut self.models[3];
        starfighter_model.translate(0.0, 0.0, MOVEMENT_SPEED * delta_time, gpu_context);
        // Hover animation
        let new_position = self.starfighter.animate(
            starfighter_model.meshes[0].instances[0].position,
            delta_time
        );
        starfighter_model.position(new_position.x, new_position.y, new_position.z, gpu_context);
        // Player controlled movement
        let pos_after_movement = self.starfighter.player_control(
            input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft),
            input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight),
            starfighter_model.meshes[0].instances[0].position,
            delta_time
        );
        starfighter_model.position(
            pos_after_movement.x,
            pos_after_movement.y,
            pos_after_movement.z,
            gpu_context
        );
    }
}

fn canyon_runner_startup(world: &mut World, ctx: &mut SystemContext) {
    let gpu = GpuContext {
        device: ctx.device.unwrap(),
        queue: ctx.queue.unwrap(),
    };

    world.add_resource(FreeCameraEnabled(false));

    let model_registry = ctx.model_registry.as_mut().unwrap();

    let starfighter_model = Starfighter::load_model(&gpu);
    let model_id = model_registry.register(starfighter_model);

    world
        .spawn()
        .with(
            Transform::new()
                .with_position(24.5, -1.0, 3.0)
                .with_scale(0.3, 0.3, 0.3)
                .with_rotation(
                    cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_y(),
                        cgmath::Deg(180.0)
                    )
                )
        )
        .with(Renderable { model_id })
        .build();
}

impl Scene for CanyonRunnerScene {
    fn update(
        &mut self,
        delta_time: f32,
        gpu_context: GpuContext,
        camera: &mut Camera,
        input: &InputState
    ) {
        // Ctrl+P toggles player movement (just_pressed prevents re-triggering every frame)
        if input.just_pressed(KeyCode::KeyP) && input.is_pressed(KeyCode::ControlLeft) {
            self.movement_enabled = !self.movement_enabled;
        }

        if self.movement_enabled {
            self.move_player(delta_time, &gpu_context, camera, input);
        }

        if input.is_pressed(KeyCode::Space) {
            let current_position = self.models[3].meshes[0].instances[0].position.clone();
            let laser_mesh = &mut self.models[4].meshes[0];
            self.laser_gun.fire(laser_mesh, current_position, &gpu_context);
        }

        // Terrain update
        let terrain_result = self.terrain_generation.terrain_update(camera.position.z);
        if let Some(new_terrain_mesh_data) = terrain_result {
            let model_to_replace = &mut self.models[self.oldest_terrain_index as usize];
            TerrainGeneration::replace_terrain_model_buffers(
                new_terrain_mesh_data,
                model_to_replace,
                &gpu_context
            );
            self.oldest_terrain_index = (self.oldest_terrain_index + 1) % 3;
        }

        // Laser update
        let laser_mesh = &mut self.models[4].meshes[0];
        self.laser_gun.update(laser_mesh, delta_time, MOVEMENT_SPEED, &gpu_context);
    }

    fn models(&self) -> &Vec<Model> {
        &self.models
    }

    fn setup_ecs(&self, schedule: &mut SystemSchedule) {
        schedule.add_startup(canyon_runner_startup);
        schedule.add_system(render_sync_system);
        schedule.add_system(camera_control_system);
    }
}

pub struct FreeCameraEnabled(pub bool);
