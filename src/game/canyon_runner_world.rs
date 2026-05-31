use cgmath::Vector3;

use crate::{
    engine::{
        assets::server::AssetServer,
        ecs::{
            component_registry::ComponentRegistry,
            components::{renderable::Renderable, transform::Transform},
            resources::{
                debug::{ShowColliderDebug, ShowDebugPanel},
                toasts::ToastQueue,
            },
            system::{SystemContext, SystemSchedule},
            world::World,
        },
        game_setup::GameSetup,
        state::context::GpuContext,
        ui::built_in::debug_panel::debug_panel,
    },
    game::{
        assets::load::load_and_register_world_models,
        components::{
            enemy::Enemy, explosion::Explosion, hover_state::HoverState, laser::Laser,
            player::Player,
        },
        difficulty::DifficultyCurve,
        events::{
            enemy_killed_event::EnemyKilledEvent, laser_fired_event::LaserFiredEvent,
            player_died_event::PlayerDiedEvent, score_event::ScoreEvent,
        },
        helpers::terrain_generation::get_initial_terrain,
        input::actions::Action,
        resources::{
            async_tasks::AsyncHighScoreTask,
            enemy_resources::EnemySpawnManager,
            game_over_state::GameOverState,
            high_scores::HighScores,
            profanity::ProfanityList,
            laser_resources::LaserManager,
            player_score::PlayerScore,
            tutorial_state::TutorialState,
            player_speed_scaling::PlayerSpeedScaling,
            screen_effects::ScreenEffects,
            terrain_resources::{TerrainGeneration, TerrainModelIds},
        },
        systems::{
            bomb_system::bomb_system,
            camera_control_system::camera_control_system,
            camera_fov_system::camera_fov_system,
            collider_debug_system::collider_debug_system,
            enemy::enemy_spawn_system::enemy_spawn_system,
            game_over_system::game_over_system,
            high_score_sync_system::{high_score_sync_system, request_fetch},
            explosion::{
                explosion_lifecycle_system::explosion_lifecycle_system,
                explosion_spawn_system::explosion_spawn_system,
            },
            hover_system::hover_system,
            laser::{laser_hit_system::laser_hit_system, laser_system::laser_system},
            pickup_collect_system::pickup_collect_system,
            pickup_glitch_system::pickup_glitch_system,
            pickup_system::pickup_system,
            player_damage_system::player_damage_system,
            player_invulnerability_system::player_invulnerability_system,
            player_score_system::player_score_system,
            player_system::player_system,
            powerup_lifecycle_system::powerup_lifecycle_system,
            score_popup_system::score_popup_system,
            screen_effects_system::screen_effects_system,
            shield_system::shield_system,
            terrain_system::terrain_system,
            tutorial_system::tutorial_system,
            vfx_system::vfx_system,
        },
        ui::{
            difficulty_debug::difficulty_debug_panel, game_over_panel::game_over_panel,
            health_indicator::health_indicator,
            pre_start_panel::pre_start_panel,
            score_counter::score_counter,
            score_popup_panel::score_popup_panel,
            toast_panel::toast_panel,
            tutorial_panel::tutorial_panel,
            white_flash_overlay::white_flash_overlay,
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
        schedule.add_game_system(camera_fov_system);
        schedule.add_game_system(hover_system);
        schedule.add_game_system(player_system);
        schedule.add_game_system(terrain_system);
        schedule.add_game_system(laser_system);
        schedule.add_game_system(enemy_spawn_system);
        schedule.add_game_system(laser_hit_system);
        schedule.add_game_system(explosion_spawn_system);
        schedule.add_game_system(pickup_system);
        schedule.add_game_system(pickup_glitch_system);
        schedule.add_game_system(pickup_collect_system);
        schedule.add_game_system(bomb_system);
        schedule.add_game_system(shield_system);
        schedule.add_game_system(powerup_lifecycle_system);
        schedule.add_game_system(player_damage_system);
        schedule.add_game_system(player_invulnerability_system);
        schedule.add_game_system(explosion_lifecycle_system);
        schedule.add_game_system(vfx_system);
        schedule.add_game_system(collider_debug_system);
        schedule.add_game_system(player_score_system);
        schedule.add_game_system(tutorial_system);
        schedule.add_game_system(score_popup_system);
        schedule.add_game_system(screen_effects_system);
        schedule.add_game_system(game_over_system);
        schedule.add_game_system(high_score_sync_system);
    }

    fn apply_design(&self, context: &egui::Context) {
        use egui_styled::prelude::*;
        context.set_styled_theme(crate::game::ui::theme::canyon_runner());
        context.set_design_data(crate::game::ui::theme::canyon_colors());
    }

    fn setup_ui(&self, ui_registry: &mut crate::engine::ui::ui_registry::UIRegistry) {
        ui_registry.add(debug_panel);
        ui_registry.add(difficulty_debug_panel);
        ui_registry.add(pre_start_panel);
        ui_registry.add(score_counter);
        ui_registry.add(score_popup_panel);
        ui_registry.add(health_indicator);
        ui_registry.add(toast_panel);
        ui_registry.add(tutorial_panel);
        ui_registry.add(white_flash_overlay);
        ui_registry.add(game_over_panel);
    }

    fn register_components(&self, registry: &mut ComponentRegistry) {
        registry.register::<HoverState>("HoverState");
        registry.register::<Player>("Player");
        registry.register::<Enemy>("Enemy");
        registry.register::<Explosion>("Explosion");
        registry.register::<Laser>("Laser");
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
        world: &mut World,
    ) {
        load_and_register_world_models(&gpu_context, asset_server, world);
    }

    fn setup(&self, world: &mut World, system_context: &mut SystemContext) {
        let gpu = GpuContext {
            device: system_context.device.unwrap(),
            queue: system_context.queue.unwrap(),
        };

        world.create_active_camera(
            gpu.device,
            Vector3::new(24.5, -0.25, 1.0),
            system_context.entity_allocator,
        );

        // Event registration
        world.register_event::<LaserFiredEvent>();
        world.register_event::<EnemyKilledEvent>();
        world.register_event::<PlayerDiedEvent>();
        world.register_event::<ScoreEvent>();

        let asset_server: &mut AssetServer = system_context.asset_server.as_mut().unwrap();

        // Debug
        world.add_resource(FreeCameraEnabled(false));
        world.add_resource(ShowDebugPanel(false));
        world.add_resource(ShowColliderDebug(false));

        // Laser setup
        world.add_resource(LaserManager::new());

        // Player setup
        world.add_resource(PlayerScore::new());
        world.add_resource(HighScores::load());
        world.add_resource(AsyncHighScoreTask::new());
        world.add_resource(GameOverState::new());
        world.add_resource(TutorialState::new());
        world.add_resource(ProfanityList::from_ron(include_str!(
            "../../assets/profanity.ron"
        )));
        world.add_resource(PlayerSpeedScaling {
            z_speed: DifficultyCurve {
                base: 10.0,
                cap: 50.0,
                warmup_score: 50.0,
                full_scale_score: 2400.0,
                exponent: 0.75,
            },
        });
        world.add_resource(ScreenEffects::new());
        world.add_resource(ToastQueue::new());

        // Enemy setup
        world.add_resource(EnemySpawnManager {
            n_enemies_spawned: 0,
            spawn_interval: DifficultyCurve {
                base: 3.0,
                cap: 1.0,
                warmup_score: 50.0,
                full_scale_score: 1700.0,
                exponent: 0.7,
            },
            time_since_last_spawn: 0.0,
            spawn_horizon_z: 80.0,
            canyon_center_x: 24.5,
            lane_offset: 0.8,
            column_z_offset: 8.0,
            despawn_behind_distance: 20.0,
            enemy_spawn_elevation: -1.0,
            enemy_spawn_scale: Vector3 {
                x: 0.3,
                y: 0.3,
                z: 0.3,
            },
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

        for model_id in terrain_model_ids {
            world
                .spawn(system_context.entity_allocator)
                .with(Renderable::new(model_id))
                .with(Transform::new())
                .build();
        }

        world.add_resource(terrain_generation);
        world.add_resource(TerrainModelIds(terrain_model_ids));

        request_fetch(world);
    }
}

pub struct FreeCameraEnabled(pub bool);
