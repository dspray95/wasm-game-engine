use crate::{engine::ecs::world::World, game::input::{actions::Action, bindings::Bindings}};

pub trait InputWorldExt {
    fn key_bindings(&self) -> Bindings<Action>;
}

impl InputWorldExt for World {
    fn key_bindings(&self) -> Bindings<Action> {
        self.get_resource::<Bindings<Action>>().unwrap().clone()
    }
}
