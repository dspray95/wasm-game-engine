use crate::engine::ecs::entity::Entity;

/// Authoritative child list, kept in sync by `World::set_parent` and the
/// despawn cascade. Used by `hierarchy_system` to walk subtrees in O(children)
/// rather than scanning every `Parent` component.
pub struct Children(pub Vec<Entity>);

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
