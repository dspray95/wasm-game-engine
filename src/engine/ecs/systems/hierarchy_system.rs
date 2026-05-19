use cgmath::{ElementWise, Vector3};

use crate::engine::ecs::{
    components::{
        children::Children, parent::Parent, transform::Transform,
        world_transform::WorldTransform,
    },
    entity::{Entity, EntityAllocator},
    system::SystemContext,
    world::World,
};

const MAX_DEPTH: u32 = 64;

/// Composes each entity's local `Transform` with its parent chain to produce a
/// world-space `WorldTransform`. Runs once per frame after all gameplay +
/// `velocity_system` writes, before any system that reads world-space data.
///
/// Three passes:
/// 1. Auto-attach `WorldTransform::identity()` to every entity that has a
///    `Transform` but no `WorldTransform` yet. Saves callers from having to
///    add it explicitly when spawning.
/// 2. Walk from roots (entities with `Transform` but no `Parent`),
///    depth-first via `Children`, composing world transforms.
/// 3. Cascade-despawn any orphans — entities with a `Parent` whose target no
///    longer exists.
pub fn hierarchy_system(world: &mut World, system_context: &mut SystemContext) {
    auto_attach_world_transforms(world, system_context.entity_allocator);

    let roots: Vec<Entity> = collect_roots(world, system_context.entity_allocator);

    for root in &roots {
        // Roots inherit identity — their WorldTransform == their Transform.
        let root_local = match world.get_component_by_id::<Transform>(root.id) {
            Some(t) => clone_transform(t),
            None => continue,
        };
        write_world_transform(world, root.id, &root_local);

        // DFS into children.
        compose_children(world, *root, &root_local, 0);
    }

    cascade_orphans(world, system_context.entity_allocator);
}

fn auto_attach_world_transforms(world: &mut World, allocator: &mut EntityAllocator) {
    let needs_attachment: Vec<u32> = world
        .iter_component::<Transform>()
        .filter(|(id, _)| world.get_component_by_id::<WorldTransform>(*id).is_none())
        .map(|(id, _)| id)
        .collect();
    for entity_id in needs_attachment {
        if let Some(entity) = allocator.lookup(entity_id) {
            world.add_component(entity, WorldTransform::identity());
        }
    }
}

fn collect_roots(world: &World, allocator: &EntityAllocator) -> Vec<Entity> {
    world
        .iter_component::<Transform>()
        .filter(|(id, _)| world.get_component_by_id::<Parent>(*id).is_none())
        .filter_map(|(id, _)| allocator.lookup(id))
        .collect()
}

fn compose_children(world: &mut World, parent: Entity, parent_world: &LocalSnapshot, depth: u32) {
    if depth >= MAX_DEPTH {
        debug_assert!(false, "hierarchy_system: exceeded max depth {} at entity {}", MAX_DEPTH, parent.id);
        log::error!(
            "hierarchy_system: exceeded max depth {} at entity {} — possible cycle",
            MAX_DEPTH,
            parent.id
        );
        return;
    }

    // Snapshot children to drop the borrow before recursing.
    let child_entities: Vec<Entity> = world
        .get_component_by_id::<Children>(parent.id)
        .map(|c| c.0.clone())
        .unwrap_or_default();

    for child in child_entities {
        let Some(child_local) = world
            .get_component_by_id::<Transform>(child.id)
            .map(clone_transform)
        else {
            continue;
        };

        let composed = LocalSnapshot {
            position: parent_world.position
                + parent_world.rotation
                    * child_local.position.mul_element_wise(parent_world.scale),
            rotation: parent_world.rotation * child_local.rotation,
            scale: parent_world.scale.mul_element_wise(child_local.scale),
        };

        write_world_transform(world, child.id, &composed);
        compose_children(world, child, &composed, depth + 1);
    }
}

fn cascade_orphans(world: &mut World, allocator: &mut EntityAllocator) {
    // An orphan is an entity with a Parent component whose target entity no
    // longer exists in the allocator (was despawned this frame or earlier
    // without going through the hierarchy-aware despawn path).
    let orphans: Vec<Entity> = world
        .iter_component::<Parent>()
        .filter_map(|(child_id, parent)| {
            if allocator.lookup(parent.0.id).is_none() {
                allocator.lookup(child_id)
            } else {
                None
            }
        })
        .collect();
    for orphan in orphans {
        world.despawn(orphan, allocator);
    }
}

/// Lightweight value-only view of Transform / WorldTransform so we can compose
/// without juggling the component borrow.
#[derive(Clone)]
struct LocalSnapshot {
    position: Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: Vector3<f32>,
}

fn clone_transform(t: &Transform) -> LocalSnapshot {
    LocalSnapshot {
        position: t.position,
        rotation: t.rotation,
        scale: t.scale,
    }
}

fn write_world_transform(world: &mut World, entity_id: u32, snapshot: &LocalSnapshot) {
    if let Some(world_transform) = world.get_component_mut_by_id::<WorldTransform>(entity_id) {
        world_transform.position = snapshot.position;
        world_transform.rotation = snapshot.rotation;
        world_transform.scale = snapshot.scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{Deg, Quaternion, Rotation3};

    use crate::engine::ecs::{
        commands::commands::Commands, entity::EntityAllocator, system::SystemContext,
    };

    fn fresh_world() -> (World, EntityAllocator) {
        let mut world = World::new();
        world.register_component::<Transform>();
        world.register_component::<WorldTransform>();
        world.register_component::<Parent>();
        world.register_component::<Children>();
        (world, EntityAllocator::default())
    }

    fn run_hierarchy(world: &mut World, allocator: &mut EntityAllocator) {
        let mut commands = Commands::new();
        let mut context = SystemContext {
            delta_time: 0.0,
            device: None,
            queue: None,
            asset_server: None,
            commands: &mut commands,
            entity_allocator: allocator,
        };
        hierarchy_system(world, &mut context);
    }

    #[test]
    fn root_world_equals_transform() {
        let (mut world, mut alloc) = fresh_world();
        let root = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(5.0, 6.0, 7.0))
            .build();
        run_hierarchy(&mut world, &mut alloc);
        let world_tf = world.get_component::<WorldTransform>(root).unwrap();
        assert!((world_tf.position.x - 5.0).abs() < 1e-5);
        assert!((world_tf.position.y - 6.0).abs() < 1e-5);
        assert!((world_tf.position.z - 7.0).abs() < 1e-5);
    }

    #[test]
    fn child_inherits_parent_translation() {
        let (mut world, mut alloc) = fresh_world();
        let parent = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(10.0, 0.0, 0.0))
            .build();
        let child = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(0.0, 5.0, 0.0))
            .as_child_of(parent)
            .build();
        run_hierarchy(&mut world, &mut alloc);
        let world_tf = world.get_component::<WorldTransform>(child).unwrap();
        assert!((world_tf.position.x - 10.0).abs() < 1e-5);
        assert!((world_tf.position.y - 5.0).abs() < 1e-5);
    }

    #[test]
    fn child_inherits_parent_scale() {
        let (mut world, mut alloc) = fresh_world();
        let parent = world
            .spawn(&mut alloc)
            .with(Transform::new().with_scale(2.0, 2.0, 2.0))
            .build();
        let child = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(1.0, 0.0, 0.0).with_scale(3.0, 3.0, 3.0))
            .as_child_of(parent)
            .build();
        run_hierarchy(&mut world, &mut alloc);
        let world_tf = world.get_component::<WorldTransform>(child).unwrap();
        // Position scaled by parent scale (2 * 1.0)
        assert!((world_tf.position.x - 2.0).abs() < 1e-5);
        // Scale multiplied (2 * 3 = 6)
        assert!((world_tf.scale.x - 6.0).abs() < 1e-5);
    }

    #[test]
    fn child_inherits_parent_rotation() {
        let (mut world, mut alloc) = fresh_world();
        let parent = world
            .spawn(&mut alloc)
            .with(Transform::new().with_rotation(Quaternion::from_angle_y(Deg(90.0))))
            .build();
        let child = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(1.0, 0.0, 0.0))
            .as_child_of(parent)
            .build();
        run_hierarchy(&mut world, &mut alloc);
        let world_tf = world.get_component::<WorldTransform>(child).unwrap();
        // Local x=1 rotated 90° around y should land at z = -1 (right-handed)
        assert!(world_tf.position.x.abs() < 1e-5);
        assert!((world_tf.position.z + 1.0).abs() < 1e-5);
    }

    #[test]
    fn two_levels_compose() {
        let (mut world, mut alloc) = fresh_world();
        let grand = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(10.0, 0.0, 0.0))
            .build();
        let parent = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(0.0, 5.0, 0.0))
            .as_child_of(grand)
            .build();
        let child = world
            .spawn(&mut alloc)
            .with(Transform::new().with_position(0.0, 0.0, 2.0))
            .as_child_of(parent)
            .build();
        run_hierarchy(&mut world, &mut alloc);
        let world_tf = world.get_component::<WorldTransform>(child).unwrap();
        assert!((world_tf.position.x - 10.0).abs() < 1e-5);
        assert!((world_tf.position.y - 5.0).abs() < 1e-5);
        assert!((world_tf.position.z - 2.0).abs() < 1e-5);
    }

    #[test]
    fn cycle_refused() {
        let (mut world, mut alloc) = fresh_world();
        let a = world.spawn(&mut alloc).with(Transform::new()).build();
        let b = world.spawn(&mut alloc).with(Transform::new()).as_child_of(a).build();
        // Try to make a a child of b — would create a cycle.
        world.set_parent(a, b);
        // Confirm a is not now a child of b (set_parent should have refused).
        assert!(world.get_component::<Parent>(a).is_none());
    }

    #[test]
    fn despawn_parent_cascades_to_children() {
        let (mut world, mut alloc) = fresh_world();
        let parent = world.spawn(&mut alloc).with(Transform::new()).build();
        let child = world.spawn(&mut alloc).with(Transform::new()).as_child_of(parent).build();
        world.despawn(parent, &mut alloc);
        // Both entities should be gone — looking them up via allocator returns None.
        assert!(alloc.lookup(parent.id).is_none());
        assert!(alloc.lookup(child.id).is_none());
    }
}
