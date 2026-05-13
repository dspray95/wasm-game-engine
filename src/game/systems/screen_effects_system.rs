use cgmath::Vector3;
use rand::Rng;

use crate::{
    engine::ecs::{components::camera::camera::Camera, system::SystemContext, world::World},
    game::{
        events::score_event::{ScoreEvent, ScoreType},
        resources::screen_effects::{ScreenEffects, SHAKE_SMALL_MAGNITUDE},
    },
};

pub fn screen_effects_system(world: &mut World, system_context: &mut SystemContext) {
    let delta_time = system_context.delta_time;

    let enemy_killed_this_frame = world
        .events::<ScoreEvent>()
        .any(|event| event.score_type == ScoreType::EnemyKilled);

    let effects = world.get_resource_mut::<ScreenEffects>().unwrap();
    effects.glitch_timer = (effects.glitch_timer - delta_time).max(0.0);
    effects.shake_timer = (effects.shake_timer - delta_time).max(0.0);
    if enemy_killed_this_frame {
        effects.trigger_kill_effect();
    }
    let shake_intensity = effects.shake_intensity();

    let active_camera_entity = world.active_camera();
    if let Some(camera) = world.get_component_mut::<Camera>(active_camera_entity) {
        if shake_intensity > 0.0 {
            let mut rng = rand::rng();
            let magnitude = SHAKE_SMALL_MAGNITUDE * shake_intensity;
            camera.shake_offset = Vector3::new(
                rng.random_range(-magnitude..magnitude),
                rng.random_range(-magnitude..magnitude),
                0.0,
            );
        } else {
            camera.shake_offset = Vector3::new(0.0, 0.0, 0.0);
        }
    }
}
