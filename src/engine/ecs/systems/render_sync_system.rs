use std::collections::HashMap;

use crate::engine::{
    ecs::{
        components::{renderable::Renderable, world_transform::WorldTransform},
        system::SystemContext,
        world::World,
    },
    instance::InstanceRaw,
};

pub fn render_sync_system(world: &mut World, system_context: &mut SystemContext) {
    let queue = system_context.queue.unwrap();
    let asset_server = system_context.asset_server.as_mut().unwrap();

    let groups = collect_instance_groups(world);

    let model_count = asset_server.models().len();
    for model_id in 0..model_count {
        match groups.get(&model_id) {
            Some(instances) => asset_server
                .get_model_mut(model_id)
                .update_instances(queue, instances),
            None => asset_server.get_model_mut(model_id).clear_instances(),
        }
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
        if !renderable.visible {
            continue;
        }
        if let Some(world_transform) = world.get_component_by_id::<WorldTransform>(entity_id) {
            groups
                .entry(renderable.model_id)
                .or_default()
                .push(world_transform.to_raw());
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::components::world_transform::WorldTransform;
    use crate::engine::ecs::entity::EntityAllocator;

    fn world_with_components() -> (World, EntityAllocator) {
        let mut world = World::new();
        world.register_component::<WorldTransform>();
        world.register_component::<Renderable>();
        (world, EntityAllocator::default())
    }

    fn world_transform_at(x: f32, y: f32, z: f32) -> WorldTransform {
        let mut t = WorldTransform::identity();
        t.position.x = x;
        t.position.y = y;
        t.position.z = z;
        t
    }

    #[test]
    fn no_entities_produces_empty_groups() {
        let (world, _alloc) = world_with_components();
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn entity_without_world_transform_is_excluded() {
        let (mut world, mut alloc) = world_with_components();
        let e = alloc.spawn();
        world.add_component(e, Renderable::new(0));
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn entity_without_renderable_is_excluded() {
        let (mut world, mut alloc) = world_with_components();
        let e = alloc.spawn();
        world.add_component(e, WorldTransform::identity());
        assert!(collect_instance_groups(&world).is_empty());
    }

    #[test]
    fn single_entity_produces_one_group_with_one_instance() {
        let (mut world, mut alloc) = world_with_components();
        let e = alloc.spawn();
        world.add_component(e, world_transform_at(1.0, 2.0, 3.0));
        world.add_component(e, Renderable::new(0));

        let groups = collect_instance_groups(&world);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[&0].len(), 1);
    }

    #[test]
    fn entities_with_same_model_id_are_grouped_together() {
        let (mut world, mut alloc) = world_with_components();
        for _ in 0..3 {
            let e = alloc.spawn();
            world.add_component(e, WorldTransform::identity());
            world.add_component(e, Renderable::new(0));
        }
        let groups = collect_instance_groups(&world);
        assert_eq!(groups[&0].len(), 3);
    }

    #[test]
    fn entities_with_different_model_ids_go_to_separate_groups() {
        let (mut world, mut alloc) = world_with_components();
        for model_id in [0, 1, 2] {
            let e = alloc.spawn();
            world.add_component(e, WorldTransform::identity());
            world.add_component(e, Renderable::new(model_id));
        }
        let groups = collect_instance_groups(&world);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[&0].len(), 1);
        assert_eq!(groups[&1].len(), 1);
        assert_eq!(groups[&2].len(), 1);
    }

    #[test]
    fn despawned_entity_is_not_included() {
        let (mut world, mut alloc) = world_with_components();
        let e = alloc.spawn();
        world.add_component(e, WorldTransform::identity());
        world.add_component(e, Renderable::new(0));
        world.despawn(e, &mut alloc);
        assert!(collect_instance_groups(&world).is_empty());
    }
}
