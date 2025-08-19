use std::vec;

use cgmath::{ vec3, Rotation3 };
use winit::{ event::ElementState, keyboard::KeyCode };

use crate::{
    engine::{
        model::{ material::Material, model::Model, vertex::ModelVertex },
        resources,
        state::context::GpuContext,
    },
    game::{
        cube,
        starfighter::{ self, Starfighter },
        terrain::Terrain,
        terrain_generation::{ self, TerrainGeneration },
    },
};

pub struct SceneManager {
    pub models: Vec<Model>,
    pub starfighter: Starfighter,
    pub terrain_generation: TerrainGeneration,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_space_pressed: bool,
}

impl SceneManager {
    pub async fn new<'a>(gpu_context: GpuContext<'a>) -> Self {
        SceneManager::load_scene(gpu_context).await.unwrap()
    }

    pub fn update(&mut self, delta_time: f32, gpu_context: GpuContext) {
        // Move forward
        let starfighter_model = &mut self.models[2];
        starfighter_model.translate(0.0, 0.0, 7.5 * delta_time, &gpu_context);
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
            _ => false,
        }
    }

    // Will load the default scene
    pub async fn load_scene<'a>(gpu_context: GpuContext<'a>) -> anyhow::Result<Self> {
        const TERRAIN_WIDTH: u32 = 50;
        const TERRAIN_LENGTH: u32 = 50;

        // Terrain setup
        let mut terrain_models: Vec<Model> = vec![];
        let terrain_generation = TerrainGeneration::new(TERRAIN_WIDTH, TERRAIN_LENGTH);
        let terrain_objects = terrain_generation.get_initial_terrain();
        for terrain_object in terrain_objects {
            let canyon_model = resources::load_model_from_arrays(
                "canyon",
                terrain_object.canyon_vertices,
                vec![],
                terrain_object.canyon_triangles,
                &gpu_context,
                Material::new([236, 95, 255], 1.0)
            );
            let terrain_model = resources::load_model_from_arrays(
                "terrain",
                terrain_object.vertices,
                vec![],
                terrain_object.triangles,
                &gpu_context,
                Material::new([60, 66, 98], 0.5)
            );
            terrain_models.push(canyon_model);
            terrain_models.push(terrain_model);
        }

        // let terrain_object = Terrain::new(TERRAIN_WIDTH, TERRAIN_LENGTH);
        // let terrain_model = resources::load_model_from_arrays(
        //     "terrain",
        //     terrain_object.vertices,
        //     vec![],
        //     terrain_object.triangles,
        //     &gpu_context,
        //     Material::new([60, 66, 98], 0.5)
        // );
        // let canyon_model = resources::load_model_from_arrays(
        //     "canyon",
        //     terrain_object.canyon_vertices,
        //     vec![],
        //     terrain_object.canyon_triangles,
        //     &gpu_context,
        //     Material::new([236, 95, 255], 1.0)
        // );

        // Player model
        let mut starfighter_model = resources
            ::load_model_from_file("starfighter.obj", &gpu_context.device).await
            .unwrap();
        starfighter_model.position(24.5, -0.5, 5.0, &gpu_context);
        starfighter_model.scale(0.35, 0.5, 0.5, &gpu_context);
        starfighter_model.rotate(
            cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_y(), // Y axis
                cgmath::Deg(180.0) // 180 degrees
            ),
            &gpu_context
        );

        let mut models: Vec<Model> = terrain_models;
        models.push(starfighter_model);

        Ok(SceneManager {
            models,
            starfighter: Starfighter::new(vec3(24.5, -0.5, 5.0)),
            terrain_generation,
            is_left_pressed: false,
            is_right_pressed: false,
            is_space_pressed: false,
        })
    }
}
