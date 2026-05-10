use crate::engine::ecs::{commands::commands::Commands, entity::Entity};

pub struct EntityCommands<'a> {
    pub entity: Entity,
    pub commands: &'a mut Commands,
}

impl<'a> EntityCommands<'a> {
    pub fn with<T: 'static>(self, component: T) -> Self {
        self.commands.add_component(self.entity, component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
