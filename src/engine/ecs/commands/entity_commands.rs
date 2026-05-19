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

    /// Queue a `set_parent` call for this entity at flush time. Mirror of
    /// `EntityBuilder::as_child_of` for the commands path.
    pub fn as_child_of(self, parent: Entity) -> Self {
        self.commands.set_parent(self.entity, parent);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
