use cgmath::{ Vector3 };

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            component_registry::ComponentRegistry,
            components::{ transform::Transform, velocity::Velocity },
            system::{ SystemContext, SystemSchedule },
            world::World,
        },
        scene::{ scene::Scene, scene_descriptor::load_scene },
        state::context::GpuContext,
    },
    game::{
        components::{ hover_state::{ HoverState }, player::Player },
        helpers::{ laser::LaserManager, starfighter, terrain_generation::get_initial_terrain },
        input::{
            actions::Action,
            bindings::Bindings,
        },
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
    asset_server.register_model("starfighter", starfighter_model);

    // For now this needs to happen _after_ we load the assets from the RON file
    load_scene_from_ron(world, asset_server);

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

fn load_scene_from_ron(world: &mut World, asset_server: &mut AssetServer) {
    let mut registry = ComponentRegistry::new();
    registry.register::<Transform>("Transform");
    registry.register::<Velocity>("Velocity");
    registry.register::<HoverState>("HoverState");
    registry.register::<Player>("Player");

    let scene_ron = include_str!("../../assets/scenes/canyon_runner.ron");
    if let Err(e) = load_scene(scene_ron, world, &registry, asset_server) {
        log::error!("Failed to load scene: {:?}", e);
    }

    let bindings_ron = include_str!("../../assets/bindings.ron");
    let bindings_descriptor = ron::from_str(bindings_ron)
        .expect("Failed to parse bindings.ron");
    world.add_resource(Bindings::<Action>::from_descriptor(bindings_descriptor));
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
