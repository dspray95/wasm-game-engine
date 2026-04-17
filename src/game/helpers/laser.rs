use web_time::Instant;
use cgmath::{ vec3, Quaternion, Rotation3 };

use crate::{
    engine::{
        instance::{ Instance, InstanceRaw },
        model::{
            descriptor::{ self, ModelDescriptor },
            material::Material,
            mesh::{ Mesh, calculate_normals },
            model::Model,
        },
        resources::{ self, load_mesh_from_arrays },
        state::context::GpuContext,
    },
    game::assets::LASER_MODEL_RON,
};

const MAX_ALIVE_LASERS: u8 = 10;
const LASER_WIDTH: f32 = 0.002;
const LASER_LENGTH: f32 = 0.05;
const LASER_Y_OFFSET: f32 = 0.05;
const LASER_SPEED: f32 = 10.0;
const FIRE_COOLDOWN_SECONDS: f32 = 0.6;
const MAX_LASER_TRAVEL: f32 = 50.0; // Define a max travel distance for lasers

pub struct LaserManager {
    pub n_live_beams: u8,
    beams: [bool; MAX_ALIVE_LASERS as usize], // Tracks which slots are active
    beams_instances: [Instance; MAX_ALIVE_LASERS as usize],
    beam_initial_z_values: [f32; MAX_ALIVE_LASERS as usize], // Keep track of where each beam was initially fired from
    current_beam_index: u8,
    last_fired_time: Instant,
}

impl LaserManager {
    pub fn new() -> Self {
        // Initialize all beams as inactive and with default instance data
        let default_instance = Instance {
            position: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
        };

        LaserManager {
            n_live_beams: 0,
            beams: [false; MAX_ALIVE_LASERS as usize],
            beams_instances: [default_instance; MAX_ALIVE_LASERS as usize],
            beam_initial_z_values: [0.0; MAX_ALIVE_LASERS as usize],
            current_beam_index: 0,
            last_fired_time: Instant::now(),
        }
    }

    pub fn load_model(gpu_context: &GpuContext) -> Model {
        let descriptor: ModelDescriptor = ron
            ::from_str(LASER_MODEL_RON)
            .expect("Failed to parse laser.ron");
        let mesh_descriptor = &descriptor.meshes[0];

        let initial_instances: Vec<Instance> = (0..MAX_ALIVE_LASERS as usize)
            .map(|_| Instance {
                position: vec3(0.0, -1000.0, 0.0),
                rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
                scale: vec3(0.0, 0.0, 0.0),
            })
            .collect();

        let mesh = load_mesh_from_arrays(
            &mesh_descriptor.label,
            mesh_descriptor.vertices
                .iter()
                .map(|(x, y, z)| [*x, *y, *z])
                .collect(),
            vec![],
            mesh_descriptor.triangles.clone(),
            gpu_context,
            Material {
                diffuse_color: [
                    mesh_descriptor.material.diffuse_color.0,
                    mesh_descriptor.material.diffuse_color.1,
                    mesh_descriptor.material.diffuse_color.2,
                ],
                alpha: mesh_descriptor.material.alpha,
            },
            Some(initial_instances),
            mesh_descriptor.max_instances
        );

        Model { meshes: vec![mesh] }
    }

    pub fn fire(
        &mut self,
        mesh: &mut Mesh,
        position: cgmath::Vector3<f32>,
        gpu_context: &GpuContext
    ) -> bool {
        // Cooldown Check
        let now = Instant::now();
        let time_since_last_fire = now.duration_since(self.last_fired_time).as_secs_f32();
        if time_since_last_fire < FIRE_COOLDOWN_SECONDS {
            return false; // Cant fire, cooldown not met
        }

        // Max Live Lasers Check
        if self.n_live_beams >= MAX_ALIVE_LASERS {
            return false;
        }

        let laser_position = vec3(position.x, position.y + LASER_Y_OFFSET, position.z);

        // Create a new Instance struct for the fired laser
        let new_instance = Instance {
            position: laser_position,
            rotation: Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), cgmath::Deg(0.0)),
            scale: vec3(10.0, 10.0, 10.0),
        };

        // Find the next available slot in our beams arrays
        let mut slot_found = false;
        for i in 0..MAX_ALIVE_LASERS as usize {
            if !self.beams[i] {
                self.beams[i] = true;
                self.beam_initial_z_values[i] = laser_position.z;
                self.beams_instances[i] = new_instance;
                self.current_beam_index = i as u8;
                self.n_live_beams += 1;
                slot_found = true;
                break;
            }
        }

        if slot_found {
            self.last_fired_time = now;
        }

        // Update buffers
        let instance_raw_data: Vec<InstanceRaw> = self.beams_instances
            .iter()
            .map(|inst| inst.to_raw())
            .collect();

        gpu_context.queue.write_buffer(
            mesh.instance_buffer.as_ref().unwrap(),
            0, // Start writing from the beginning of the buffer
            bytemuck::cast_slice(&instance_raw_data)
        );
        slot_found
    }

    pub fn update(
        &mut self,
        mesh: &mut Mesh,
        delta_time: f32,
        current_speed: f32,
        gpu_context: &GpuContext
    ) {
        for i in 0..MAX_ALIVE_LASERS as usize {
            if self.beams[i] {
                // If the beam is active, update its position along the Z-axis
                self.beams_instances[i].position.z += (LASER_SPEED + current_speed) * delta_time;

                if
                    self.beams_instances[i].position.z >
                    self.beam_initial_z_values[i] + MAX_LASER_TRAVEL
                {
                    self.beams[i] = false;
                    self.n_live_beams -= 1;
                    self.beams_instances[i].position = vec3(0.0, 0.0, 0.0);
                }
            }
        }

        let instance_raw_data: Vec<InstanceRaw> = self.beams_instances
            .iter()
            .map(|inst| inst.to_raw())
            .collect();

        gpu_context.queue.write_buffer(
            mesh.instance_buffer.as_ref().unwrap(),
            0, // Start writing from the beginning of the buffer
            bytemuck::cast_slice(&instance_raw_data)
        );
    }
}
