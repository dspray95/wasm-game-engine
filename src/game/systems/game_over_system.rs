use cgmath::Vector3;

use crate::{
    engine::ecs::{
        components::{renderable::Renderable, transform::Transform, velocity::Velocity},
        resources::{camera::ActiveCamera, toasts::ToastQueue},
        system::SystemContext,
        world::World,
    },
    game::{
        systems::high_score_sync_system::request_fetch,
        components::{
            bomb_charging::BombCharging,
            dead::Dead,
            double_fire_rate::DoubleFireRate,
            enemy::Enemy,
            explosion::Explosion,
            glitch_vfx::GlitchVfx,
            hyperdrive::Hyperdrive,
            hyperdrive_pickup_glitch::HyperdrivePickupGlitch,
            invulnerable::Invulnerable,
            laser::Laser,
            pickup::Pickup,
            player::Player,
            shield::Shield,
        },
        resources::{
            enemy_resources::EnemySpawnManager,
            game_over_state::{GameOverPhase, GameOverState},
            high_scores::HighScores,
            laser_resources::LaserManager,
            player_score::PlayerScore,
            screen_effects::ScreenEffects,
        },
    },
};

const PLAYER_SPAWN_X: f32 = 24.5;
const PLAYER_SPAWN_Y: f32 = -1.0;
// Offset ahead of the camera. The original scene placed camera at z=1.0 and
// player at z=3.0, so the camera trails the player by 2 units.
const PLAYER_Z_AHEAD_OF_CAMERA: f32 = 2.0;
const PLAYER_MAX_HEALTH: i32 = 3;

pub fn game_over_system(world: &mut World, system_context: &mut SystemContext) {
    let player_entity = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next();

    let phase = world
        .get_resource::<GameOverState>()
        .map(|state| state.phase)
        .unwrap_or(GameOverPhase::Playing);

    match phase {
        GameOverPhase::PreStart => {
            return;
        }
        GameOverPhase::Playing => {
            if let Some(entity) = player_entity {
                if world.get_component::<Dead>(entity).is_some() {
                    let score = world
                        .get_resource::<PlayerScore>()
                        .map(|s| s.score)
                        .unwrap_or(0);
                    if let Some(state) = world.get_resource_mut::<GameOverState>() {
                        state.phase = GameOverPhase::DeathRamping;
                        state.final_score = score;
                    }
                }
            }
        }
        GameOverPhase::DeathRamping => {
            let ramp_done = player_entity
                .and_then(|entity| world.get_component::<Dead>(entity))
                .map(|dead| dead.ramp_time_remaining <= 0.0)
                .unwrap_or(true);
            if ramp_done {
                let qualifies = {
                    let score = world
                        .get_resource::<GameOverState>()
                        .map(|s| s.final_score)
                        .unwrap_or(0);
                    world
                        .get_resource::<HighScores>()
                        .map(|hs| hs.qualifies(score))
                        .unwrap_or(false)
                };
                if let Some(state) = world.get_resource_mut::<GameOverState>() {
                    state.phase = if qualifies {
                        GameOverPhase::EnteringInitials
                    } else {
                        GameOverPhase::Showing
                    };
                }
                request_fetch(world);
            }
        }
        GameOverPhase::EnteringInitials | GameOverPhase::Showing => {
            let restart = world
                .get_resource::<GameOverState>()
                .map(|s| s.restart_requested)
                .unwrap_or(false);
            if restart {
                perform_reset(world, system_context);
            }
        }
    }
}

fn perform_reset(world: &mut World, system_context: &mut SystemContext) {
    // Despawn transient gameplay entities. Terrain keeps scrolling — this is
    // an endless runner, the player just gets to start enemy spawning fresh.
    let mut to_despawn: Vec<_> = world
        .get_entities_with::<Enemy>(system_context.entity_allocator);
    to_despawn.extend(world.get_entities_with::<Laser>(system_context.entity_allocator));
    to_despawn.extend(world.get_entities_with::<Explosion>(system_context.entity_allocator));
    to_despawn.extend(world.get_entities_with::<Pickup>(system_context.entity_allocator));
    to_despawn.extend(world.get_entities_with::<GlitchVfx>(system_context.entity_allocator));
    to_despawn
        .extend(world.get_entities_with::<HyperdrivePickupGlitch>(system_context.entity_allocator));
    for entity in to_despawn {
        world.despawn(entity, system_context.entity_allocator);
    }

    // Respawn position rides on the camera so terrain (which doesn't reset)
    // stays aligned and enemy spawning, which is camera-relative, still
    // produces enemies in front of the player.
    let camera_z = world
        .get_resource::<ActiveCamera>()
        .map(|active_camera| active_camera.0)
        .and_then(|entity| world.get_component::<Transform>(entity))
        .map(|transform| transform.position.z)
        .unwrap_or(1.0);
    let spawn_position = Vector3 {
        x: PLAYER_SPAWN_X,
        y: PLAYER_SPAWN_Y,
        z: camera_z + PLAYER_Z_AHEAD_OF_CAMERA,
    };

    // Reset the player entity in place (it survived the despawn pass — the
    // sparse-set lookup above only finds entities with the listed components).
    let player_entity = world
        .get_entities_with::<Player>(system_context.entity_allocator)
        .into_iter()
        .next();
    if let Some(entity) = player_entity {
        world.remove_component::<Dead>(entity);
        world.remove_component::<Invulnerable>(entity);
        world.remove_component::<Shield>(entity);
        world.remove_component::<Hyperdrive>(entity);
        world.remove_component::<DoubleFireRate>(entity);
        world.remove_component::<BombCharging>(entity);
        if let Some(player) = world.get_component_mut::<Player>(entity) {
            player.health = PLAYER_MAX_HEALTH;
            player.move_player = true;
        }
        if let Some(renderable) = world.get_component_mut::<Renderable>(entity) {
            renderable.visible = true;
        }
        if let Some(transform) = world.get_component_mut::<Transform>(entity) {
            transform.position = spawn_position;
        }
        if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
            velocity.x = 0.0;
            velocity.y = 0.0;
            velocity.z = 0.0;
        }
    }

    // Reset gameplay resources.
    if let Some(score) = world.get_resource_mut::<PlayerScore>() {
        *score = PlayerScore::new();
    }
    if let Some(effects) = world.get_resource_mut::<ScreenEffects>() {
        *effects = ScreenEffects::new();
    }
    if let Some(toasts) = world.get_resource_mut::<ToastQueue>() {
        toasts.toasts.clear();
    }
    if let Some(spawner) = world.get_resource_mut::<EnemySpawnManager>() {
        spawner.n_enemies_spawned = 0;
        spawner.time_since_last_spawn = 0.0;
        spawner.enemy_entities.clear();
    }
    if let Some(lasers) = world.get_resource_mut::<LaserManager>() {
        *lasers = LaserManager::new();
    }

    if let Some(state) = world.get_resource_mut::<GameOverState>() {
        state.reset();
    }
}
