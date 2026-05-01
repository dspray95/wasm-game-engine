use cgmath::{ Vector3 };

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            component_registry::ComponentRegistry,
            resources::debug::{ ShowColliderDebug, ShowDebugPanel },
            system::{ SystemContext, SystemSchedule },
            world::World,
        },
        game_setup::GameSetup,
        state::context::GpuContext,
        ui::built_in::debug_panel::debug_panel,
    },
    game::{
        assets::load::load_and_register_world_models,
        components::{ hover_state::HoverState, player::Player },
        events::laser_fired_event::LaserFiredEvent,
        helpers::{ laser::LaserManager, terrain_generation::get_initial_terrain },
        input::actions::Action,
        resources::{
            enemy_resources::EnemySpawnManager,
            move_player::MovePlayer,
            terrain_resources::{ TerrainGeneration, TerrainModelIds },
        },
        systems::{
            camera_control_system::camera_control_system,
            collider_debug_system::collider_debug_system,
            collision_log_system::collision_log_system,
            enemy_spawn_system::enemy_spawn_system,
            hover_system::hover_system,
            laser_log_system::laser_log_system,
            laser_system::laser_system,
            player_system::player_system,
            terrain_system::terrain_system,
        },
    },
};

const TERRAIN_WIDTH: u32 = 50;
const TERRAIN_LENGTH: u32 = 150;

pub struct CanyonRunnerWorld;

impl GameSetup for CanyonRunnerWorld {
    type Action = Action;

    fn setup_ecs(&self, schedule: &mut SystemSchedule) {
        schedule.add_game_system(camera_control_system);
        schedule.add_game_system(hover_system);
        schedule.add_game_system(player_system);
        schedule.add_game_system(terrain_system);
        schedule.add_game_system(laser_system);
        schedule.add_game_system(laser_log_system);
        schedule.add_game_system(collision_log_system);
        schedule.add_game_system(enemy_spawn_system);
        schedule.add_game_system(collider_debug_system);
    }

    fn setup_ui(&self, ui_registry: &mut crate::engine::ui::ui_registry::UIRegistry) {
        ui_registry.add(debug_panel);
    }

    fn register_components(&self, registry: &mut ComponentRegistry) {
        registry.register::<HoverState>("HoverState");
        registry.register::<Player>("Player");
    }

    fn world_ron(&self) -> Option<&'static str> {
        Some(include_str!("../../assets/worlds/canyon_runner.ron"))
    }

    fn bindings_ron(&self) -> Option<&'static str> {
        Some(include_str!("../../assets/bindings.ron"))
    }

    fn load_assets(
        &self,
        gpu_context: &GpuContext,
        asset_server: &mut AssetServer,
        world: &mut World
    ) {
        load_and_register_world_models(&gpu_context, asset_server, world);
    }

    fn setup(&self, world: &mut World, system_context: &mut SystemContext) {
        let gpu = GpuContext {
            device: system_context.device.unwrap(),
            queue: system_context.queue.unwrap(),
        };

        world.create_active_camera(gpu.device, Vector3::new(24.5, -0.25, 1.0));
        world.add_resource(FreeCameraEnabled(false));
        world.add_resource(ShowDebugPanel(false));
        world.add_resource(ShowColliderDebug(false));
        world.register_event::<LaserFiredEvent>();

        let asset_server: &mut AssetServer = system_context.asset_server.as_mut().unwrap();

        // Laser setup
        world.add_resource(LaserManager::new());

        // Player setup
        world.add_resource(MovePlayer(true));

        // Enemy setup
        world.add_resource(EnemySpawnManager {
            n_enemies_spawned: 0,
            z_gap_between_spanws: 100.0,
            last_z_pos_spawned_at: 0.0,
            canyon_center_x: 24.5,
            enemy_spawn_elevation: -1.0,
            enemy_spawn_scale: Vector3 { x: 0.3, y: 0.3, z: 0.3 },
            enemy_entities: Vec::new(),
        });
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
}

pub struct FreeCameraEnabled(pub bool);
