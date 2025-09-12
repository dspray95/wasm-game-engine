use cgmath::{ vec3, Vector3 };
use wgpu::core::device::resource;

use crate::engine::{
    model::{ material::Material, mesh::{ calculate_normals, MeshData }, model::Model },
    resources,
    state::context::GpuContext,
};

pub struct Starfighter {
    current_direction: String,
    upper_limit: f32,
    lower_limit: f32,
    left_limit: f32,
    right_limit: f32,
}

impl Starfighter {
    pub fn new(start_position: Vector3<f32>) -> Starfighter {
        Starfighter {
            current_direction: "up".to_string(),
            upper_limit: -0.9,
            lower_limit: -0.99,
            left_limit: start_position.x + 1.0,
            right_limit: start_position.x - 1.0,
        }
    }

    pub fn player_control(
        &mut self,
        is_left_pressed: bool,
        is_right_pressed: bool,
        current_position: Vector3<f32>,
        delta_time: f32
    ) -> Vector3<f32> {
        let movement_speed = 4.0;

        let x_movement = {
            if is_left_pressed {
                movement_speed * delta_time
            } else if is_right_pressed {
                -movement_speed * delta_time
            } else {
                0.0
            }
        };

        let new_x_pos = (current_position.x + x_movement)
            .max(self.right_limit)
            .min(self.left_limit);

        vec3(new_x_pos, current_position.y, current_position.z)
    }

    pub fn animate(&mut self, current_position: Vector3<f32>, delta_time: f32) -> Vector3<f32> {
        let mut new_position = current_position.clone();
        if self.current_direction == "up" {
            new_position.y = current_position.y + 0.2 * delta_time;
            if new_position.y >= self.upper_limit {
                self.current_direction = "down".to_string();
            }
        } else {
            new_position.y = current_position.y - 0.2 * delta_time;
            if new_position.y <= self.lower_limit {
                self.current_direction = "up".to_string();
            }
        }

        new_position
    }

    pub fn load_model(gpu_context: &GpuContext) -> Model {
        let fuselage_vertices = vec![
            [0.000001, -0.0, -1.853511],
            [0.35669, -0.0, -1.140133],
            [0.000001, 0.142676, -1.140133],
            [0.285351, 0.142676, 0.286622],
            [0.000001, 0.356689, -0.070067],
            [0.0, -0.0, 0.643311],
            [1.070067, -0.0, 1.0],
            [-0.356688, -0.0, -1.140133],
            [-1.070067, 0.0, 0.999999],
            [-0.285351, 0.142676, 0.286622]
        ];

        let fuselage_triangles = vec![
            0,
            2,
            1,
            2,
            3,
            1,
            4,
            5,
            3,
            1,
            3,
            6,
            7,
            2,
            0,
            7,
            9,
            2,
            5,
            6,
            3,
            9,
            5,
            4,
            8,
            5,
            9,
            8,
            9,
            7
        ];

        let fuselage_mesh_data = MeshData {
            normals: calculate_normals(&fuselage_vertices, &fuselage_triangles),
            vertices: fuselage_vertices,
            triangles: fuselage_triangles,
            material: Material {
                diffuse_color: [0, 255, 255],
                alpha: 1.0,
            },
        };

        let cockpit_vertices = vec![
            [0.000001, 0.142676, -1.140133],
            [0.285351, 0.142676, 0.286622],
            [0.000001, 0.356689, -0.070067],
            [-0.285351, 0.142676, 0.286622]
        ];

        let cockpit_triangles = vec![0, 2, 1, 0, 3, 2];
        let cockpit_mesh_data = MeshData {
            normals: calculate_normals(&cockpit_vertices, &cockpit_triangles),
            vertices: cockpit_vertices,
            triangles: cockpit_triangles,
            material: Material {
                diffuse_color: [0, 0, 0],
                alpha: 1.0,
            },
        };

        let fuselage_mesh = resources::load_mesh_from_arrays(
            "Starfighter Fuselage",
            fuselage_mesh_data.vertices,
            fuselage_mesh_data.normals,
            fuselage_mesh_data.triangles,
            &gpu_context,
            fuselage_mesh_data.material,
            None
        );

        let cockpit_mesh = resources::load_mesh_from_arrays(
            "Starfighter Cockpit",
            cockpit_mesh_data.vertices,
            cockpit_mesh_data.normals,
            cockpit_mesh_data.triangles,
            gpu_context,
            cockpit_mesh_data.material,
            None
        );

        Model {
            meshes: vec![fuselage_mesh, cockpit_mesh],
        }
    }
}
