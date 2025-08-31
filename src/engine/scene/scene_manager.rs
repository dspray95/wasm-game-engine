use std::vec;

use cgmath::{ vec3, Rotation3 };
use winit::{ event::ElementState, keyboard::KeyCode };

use crate::{
    engine::{
        camera::camera::Camera,
        model::{ material::Material, model::Model },
        resources,
        state::context::GpuContext,
    },
    game::{ starfighter::Starfighter, terrain::TerrainGeneration },
};

const TERRAIN_WIDTH: u32 = 50;
const TERRAIN_LENGTH: u32 = 150;

pub struct SceneManager {
    pub models: Vec<Model>, // 0-5 are terrain, 6 is the player
    pub starfighter: Starfighter,
    pub terrain_generation: TerrainGeneration,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_space_pressed: bool,
    pub oldest_terrain_index: u32,
    is_control_pressed: bool,
    is_p_pressed: bool,
    movement_enabled: bool,
}

impl SceneManager {
    pub async fn new<'a>(gpu_context: GpuContext<'a>) -> Self {
        SceneManager::load_scene(gpu_context).await.unwrap()
    }

    pub fn update(&mut self, delta_time: f32, gpu_context: GpuContext, camera: &mut Camera) {
        if self.movement_enabled {
            self.move_player(delta_time, &gpu_context, camera);
            if self.is_control_pressed && self.is_p_pressed {
                self.movement_enabled = false;
            }
        } else if self.is_control_pressed && self.is_p_pressed {
            self.movement_enabled = true;
        }

        let terrain_result = self.terrain_generation.terrain_update(camera.position.z);

        if let Some(new_terrain_mesh_data) = terrain_result {
            log::info!("Replacing terrain at index: {}", self.oldest_terrain_index);

            let model_to_replace = &mut self.models[self.oldest_terrain_index as usize];
            TerrainGeneration::replace_terrain_model_buffers(
                new_terrain_mesh_data,
                model_to_replace,
                &gpu_context
            );
            self.oldest_terrain_index = (self.oldest_terrain_index + 1) % 3;
        }
    }

    fn move_player(&mut self, delta_time: f32, gpu_context: &GpuContext, camera: &mut Camera) {
        self.move_camera(delta_time, gpu_context, camera);
        self.move_starfighter(delta_time, gpu_context);
    }

    fn move_camera(&mut self, delta_time: f32, gpu_context: &GpuContext, camera: &mut Camera) {
        camera.translate(0.0, 0.0, 20.0 * delta_time, gpu_context.queue);
    }

    fn move_starfighter(&mut self, delta_time: f32, gpu_context: &GpuContext) {
        // Move forward
        let starfighter_model = &mut self.models[3];
        starfighter_model.translate(0.0, 0.0, 20.0 * delta_time, &gpu_context);
        // Hover animation
        let new_position = self.starfighter.animate(
            starfighter_model.meshes[0].instances[0].position,
            delta_time
        );
        starfighter_model.position(new_position.x, new_position.y, new_position.z, &gpu_context);
        // Player controlled movement
        let pos_after_movement = self.starfighter.player_control(
            self.is_left_pressed,
            self.is_right_pressed,
            starfighter_model.meshes[0].instances[0].position,
            delta_time
        );
        starfighter_model.position(
            pos_after_movement.x,
            pos_after_movement.y,
            pos_after_movement.z,
            &gpu_context
        );
    }

    pub fn player_control_event(&mut self, key_code: KeyCode, key_state: ElementState) -> bool {
        let is_pressed = key_state == ElementState::Pressed;

        match key_code {
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_space_pressed = is_pressed;
                true
            }
            KeyCode::ControlLeft => {
                self.is_control_pressed = is_pressed;
                true
            }
            KeyCode::KeyP => {
                self.is_p_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    // Will load the default scene
    pub async fn load_scene<'a>(gpu_context: GpuContext<'a>) -> anyhow::Result<Self> {
        // Terrain setup
        let mut terrain_generation = TerrainGeneration::new(TERRAIN_WIDTH, TERRAIN_LENGTH);
        let terrain_models = terrain_generation.get_initial_terrain(&gpu_context);

        // Player model
        let mut starfighter_model = resources
            ::load_model_from_file("starfighter.obj", &gpu_context.device).await
            .unwrap();
        starfighter_model.position(24.5, -1.0, 3.0, &gpu_context);
        starfighter_model.scale(0.3, 0.3, 0.3, &gpu_context);
        starfighter_model.rotate(
            cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_y(), // Y axis
                cgmath::Deg(180.0) // 180 degrees
            ),
            &gpu_context
        );

        let mut models: Vec<Model> = terrain_models.into();
        models.push(starfighter_model);

        Ok(SceneManager {
            models,
            starfighter: Starfighter::new(vec3(24.5, -0.5, 5.0)),
            terrain_generation,
            is_left_pressed: false,
            is_right_pressed: false,
            is_space_pressed: false,
            oldest_terrain_index: 0,
            is_control_pressed: false,
            is_p_pressed: false,
            movement_enabled: true,
        })
    }
}
