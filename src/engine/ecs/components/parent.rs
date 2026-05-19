use crate::engine::ecs::entity::Entity;

/// Marks an entity as a child of another. The hierarchy_system composes the
/// child's local Transform with the parent's WorldTransform each frame.
///
/// Maintain via `World::set_parent` / `Commands::set_parent` /
/// `EntityBuilder::as_child_of` — those keep the parent's `Children` list in
/// sync. Inserting `Parent` directly will work but leaves `Children`
/// inconsistent.
#[derive(Clone, Copy)]
pub struct Parent(pub Entity);
