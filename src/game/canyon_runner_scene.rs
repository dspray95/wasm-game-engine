use cgmath::{ Rotation3, Vector3 };

use crate::{
    engine::{
        ecs::{
            components::{ renderable::Renderable, transform::Transform, velocity::Velocity },
            system::{ SystemContext, SystemSchedule },
            world::World,
        },
        scene::scene::Scene,
        state::context::GpuContext,
    },
    game::{
        components::{ hover_state::{ HoverDirection, HoverState }, player::Player },
        helpers::{ laser::LaserManager, starfighter, terrain_generation::get_initial_terrain },
        resources::{
            laser_resources::LaserModelId,
            terrain_resources::{ TerrainGeneration, TerrainModelIds },
        },
        systems::{
            camera_control_system::camera_control_system,
            hover_system::hover_system,
            laser_system::laser_system,
            player_system::player_system,
            terrain_system::terrain_system,
        },
    },
};

const TERRAIN_WIDTH: u32 = 50;
const TERRAIN_LENGTH: u32 = 150;

pub struct CanyonRunnerScene;

fn canyon_runner_startup(world: &mut World, system_context: &mut SystemContext) {
    let gpu = GpuContext {
        device: system_context.device.unwrap(),
        queue: system_context.queue.unwrap(),
    };

    world.create_active_camera(gpu.device, Vector3::new(24.5, -0.25, 1.0));
    world.add_resource(FreeCameraEnabled(false));

    let asset_server = system_context.asset_server.as_mut().unwrap();

    // Player setup
    let starfighter_model = starfighter::load_model(&gpu);
    let starfighter_model_id = asset_server.register_model("starfighter", starfighter_model);

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
        .with(Velocity { x: 0.0, y: 0.0, z: 0.0 })
        .with(Renderable { model_id: starfighter_model_id })
        .with(HoverState { direction: HoverDirection::Up, upper_limit: -0.9, lower_limit: -0.99 })
        .with(Player {})
        .build();

    // Laser setup
    let laser_model = LaserManager::load_model(&gpu);
    let laser_model_id = asset_server.register_model("laser", laser_model);
    world.add_resource(LaserModelId(laser_model_id));
    world.add_resource(LaserManager::new());

    // Terrain setup
    let mut terrain_generation = TerrainGeneration {
        terrain_width: TERRAIN_WIDTH,
        terrain_length: TERRAIN_LENGTH,
        n_chunks_generated: 0,
        next_breakpoint: 0.0,
        oldest_terrain_index: 0,
    };

    let [terrain_a, terrain_b, terrain_c] = get_initial_terrain(&mut terrain_generation, &gpu);
    let terrain_model_ids = [
        asset_server.register_model("terrain_a", terrain_a),
        asset_server.register_model("terrain_b", terrain_b),
        asset_server.register_model("terrain_c", terrain_c),
    ];

    world.add_resource(terrain_generation);
    world.add_resource(TerrainModelIds(terrain_model_ids));
}

impl Scene for CanyonRunnerScene {
    fn setup_ecs(&self, schedule: &mut SystemSchedule) {
        schedule.add_startup(canyon_runner_startup);
        schedule.add_game_system(camera_control_system);
        schedule.add_game_system(hover_system);
        schedule.add_game_system(player_system);
        schedule.add_game_system(terrain_system);
        schedule.add_game_system(laser_system);
    }
}

pub struct FreeCameraEnabled(pub bool);
