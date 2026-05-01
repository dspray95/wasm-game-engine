use cgmath::Vector3;

use crate::{
    engine::{
        ecs::{ components::transform::Transform, system::SystemContext, world::World },
        events::events::Events,
        state::context::GpuContext,
    },
    game::{
        components::player::Player,
        events::laser_fired_event::LaserFiredEvent,
        helpers::laser::LaserManager,
        input::{ actions::Action, world_ext::InputWorldExt },
    },
};

const MOVEMENT_SPEED: f32 = 10.0;

pub fn laser_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    let player_position = world
        .query_iter::<(&Player, &Transform)>()
        .next()
        .map(|(_, transform)| transform.position);

    let laser_model_id = system_context.asset_server.as_deref().unwrap().get_model_id("laser");
    let delta_time = system_context.delta_time;

    let gpu = GpuContext {
        device: system_context.device.unwrap(),
        queue: system_context.queue.unwrap(),
    };

    let mesh = system_context.asset_server
        .as_deref_mut()
        .unwrap()
        .get_model_mut(laser_model_id)
        .meshes.get_mut(0)
        .unwrap();

    let tried_to_fire = key_bindings.is_action_pressed(&Action::Fire, &input);

    let fired_from: Option<Vector3<f32>> = {
        let laser_manager = world.get_resource_mut::<LaserManager>().unwrap();
        let result = match (tried_to_fire, player_position) {
            (true, Some(pos)) if laser_manager.fire(mesh, pos, &gpu) => Some(pos),
            _ => None,
        };
        laser_manager.update(mesh, delta_time, MOVEMENT_SPEED, &gpu);
        result
    };

    if let Some(position) = fired_from {
        world
            .get_resource_mut::<Events<LaserFiredEvent>>()
            .unwrap()
            .send(LaserFiredEvent { origin: position });
    }
}
