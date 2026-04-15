use crate::{
    engine::ecs::{
        components::{ transform::Transform, velocity::Velocity },
        resources::input_state::InputState,
        system::SystemContext,
        world::World,
    },
    game::components::player::Player,
};

pub fn player_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.get_resource::<InputState>().unwrap().clone();

    let player_id = world.iter_component::<Player>().next().unwrap().0;

    let (player_transform, player_velocity) = world
        .query::<(&mut Transform, &mut Velocity)>(player_id)
        .unwrap();
}
