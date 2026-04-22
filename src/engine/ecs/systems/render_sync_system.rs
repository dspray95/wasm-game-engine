use std::collections::HashMap;

use crate::engine::{
    ecs::{
        components::{ renderable::Renderable, transform::Transform },
        system::SystemContext,
        world::World,
    },
    instance::InstanceRaw,
};

pub fn render_sync_system(world: &mut World, system_context: &mut SystemContext) {
    let queue = system_context.queue.unwrap();
    let asset_server = system_context.asset_server.as_mut().unwrap();

    let groups = collect_instance_groups(world);

    for (model_id, instances) in &groups {
        asset_server.get_model_mut(*model_id).update_instances(queue, instances);
    }
}

// Groups InstanceRaw data by model_id for all entities with both Transform and Renderable.
//
// PERFORMANCE NOTES (acceptable at current scale, revisit when profiler says so):
//
// 1. HashMap allocation - a new HashMap and Vec<InstanceRaw> per model group is heap-allocated
//    every frame. Fix: keep a persistent HashMap<usize, Vec<InstanceRaw>> as a resource,
//    call clear() each frame to reuse the allocation rather than dropping and recreating it.
//
// 2. Double iteration - we iterate Renderable to collect entity IDs, then look up Transform
//    for each one (two sparse set reads per entity). This is O(n) but with a constant factor.
//    An archetype-based ECS stores entities with the same component set contiguously, making
//    this a single pass. That's the main tradeoff of sparse sets vs archetypes.
//
// 3. write_buffer every frame - instance data is uploaded to the GPU unconditionally, even
//    for static models that haven't moved. Fix: add dirty: Vec<bool> + any_dirty: bool to
//    SparseSet<T>, set on get_mut(), check in render_sync before uploading. Static buildings
//    would then pay zero upload cost after initial placement.
fn collect_instance_groups(world: &World) -> HashMap<usize, Vec<InstanceRaw>> {
    let mut groups: HashMap<usize, Vec<InstanceRaw>> = HashMap::new();

    for (entity_id, renderable) in world.iter_component::<Renderable>() {
        if let Some(transform) = world.get_component_by_id::<Transform>(entity_id) {
            groups.entry(renderable.model_id).or_default().push(transform.to_raw());
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::components::transform::Transform;

    fn world_with_components() -> World {
        let mut world = World::new();
        world.register_component::<Transform>();
        world.register_component::<Renderable>();
        world
    }

    #[test]
    fn no_entities_produces_empty_groups() {
        let world = world_with_components();
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn entity_without_transform_is_excluded() {
        let mut world = world_with_components();
        let e = world.spawn_entity_only();
        world.add_component(e, Renderable { model_id: 0 });
        // No Transform added — should not appear in groups
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn entity_without_renderable_is_excluded() {
        let mut world = world_with_components();
        let e = world.spawn_entity_only();
        world.add_component(e, Transform::new());
        // No Renderable added — should not appear in groups
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn single_entity_produces_one_group_with_one_instance() {
        let mut world = world_with_components();
        let e = world.spawn_entity_only();
        world.add_component(e, Transform::new().with_position(1.0, 2.0, 3.0));
        world.add_component(e, Renderable { model_id: 0 });

        let groups = collect_instance_groups(&world);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[&0].len(), 1);
    }

    #[test]
    fn entities_with_same_model_id_are_grouped_together() {
        let mut world = world_with_components();
        for _ in 0..3 {
            let e = world.spawn_entity_only();
            world.add_component(e, Transform::new());
            world.add_component(e, Renderable { model_id: 0 });
        }
        let groups = collect_instance_groups(&world);
        assert_eq!(groups[&0].len(), 3);
    }

    #[test]
    fn entities_with_different_model_ids_go_to_separate_groups() {
        let mut world = world_with_components();
        for model_id in [0, 1, 2] {
            let e = world.spawn_entity_only();
            world.add_component(e, Transform::new());
            world.add_component(e, Renderable { model_id });
        }
        let groups = collect_instance_groups(&world);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[&0].len(), 1);
        assert_eq!(groups[&1].len(), 1);
        assert_eq!(groups[&2].len(), 1);
    }

    #[test]
    fn despawned_entity_is_not_included() {
        let mut world = world_with_components();
        let e = world.spawn_entity_only();
        world.add_component(e, Transform::new());
        world.add_component(e, Renderable { model_id: 0 });
        world.despawn(e);
        assert!(collect_instance_groups(&world).is_empty());
    }
}
