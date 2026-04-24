use winit::keyboard::KeyCode;

use crate::{
    engine::{
        ecs::{
            components::transform::Transform,
            system::SystemContext,
            world::World,
        },
        input::input_state::InputState,
        state::context::GpuContext,
    },
    game::{
        components::player::Player,
        helpers::laser::LaserManager,
        resources::laser_resources::LaserModelId,
    },
};

const MOVEMENT_SPEED: f32 = 10.0;

pub fn laser_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    let player_position = world
        .query_iter::<(&Player, &Transform)>()
        .next()
        .map(|(_, transform)| transform.position);

    let laser_model_id = world.get_resource::<LaserModelId>().unwrap().0;
    let delta_time = system_context.delta_time;

    let gpu = GpuContext {
        device: system_context.device.unwrap(),
        queue: system_context.queue.unwrap(),
    };

    let mesh = system_context.asset_server
        .as_mut()
        .unwrap()
        .get_model_mut(laser_model_id)
        .meshes.get_mut(0)
        .unwrap();

    let laser_manager = world.get_resource_mut::<LaserManager>().unwrap();

    if input.is_pressed(KeyCode::Space) {
        if let Some(position) = player_position {
            laser_manager.fire(mesh, position, &gpu);
        }
    }

    laser_manager.update(mesh, delta_time, MOVEMENT_SPEED, &gpu);
}
