use cgmath::{ One, Quaternion, Vector3 };

use crate::{
    engine::ecs::{
        components::{
            collider::{ Collider, ColliderShape },
            renderable::Renderable,
            transform::Transform,
        },
        resources::debug::{ DebugVisual, ShowColliderDebug },
        system::SystemContext,
        world::World,
    },
    game::input::{ actions::Action, world_ext::InputWorldExt },
};

pub fn collider_debug_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    if key_bindings.is_action_just_pressed(&Action::ToggleColliderDebug, &input) {
        let resource = world.get_resource_mut::<ShowColliderDebug>().unwrap();
        resource.0 = !resource.0;
    }

    let show = world.get_resource::<ShowColliderDebug>().map(|r| r.0).unwrap_or(false);

    let stale_visual_ids: Vec<u32> = world.get_entities_with::<DebugVisual>();
    for entity_id in stale_visual_ids {
        if let Some(entity) = world.get_entity(entity_id) {
            world.despawn(entity);
        }
    }

    if !show {
        return;
    }

    let cube_model_id = system_context
        .asset_server
        .as_deref()
        .unwrap()
        .get_model_id("cube");

    let collider_snapshots: Vec<(u32, ColliderShape)> = world
        .iter_component::<Collider>()
        .map(|(entity_id, collider)| (entity_id, collider.shape.clone()))
        .collect();

    let visuals: Vec<(Vector3<f32>, Vector3<f32>)> = collider_snapshots
        .into_iter()
        .filter_map(|(entity_id, shape)| {
            let transform = world.get_component_by_id::<Transform>(entity_id)?;
            let (offset, half_extents) = match shape {
                ColliderShape::AABB { offset, half_extents } => (
                    offset,
                    Vector3::new(
                        half_extents.x * transform.scale.x,
                        half_extents.y * transform.scale.y,
                        half_extents.z * transform.scale.z,
                    ),
                ),
                ColliderShape::Sphere { offset, radius } => {
                    let r = radius * transform.scale.x.max(transform.scale.y).max(transform.scale.z);
                    (offset, Vector3::new(r, r, r))
                }
            };
            let scaled_offset = Vector3::new(
                offset.x * transform.scale.x,
                offset.y * transform.scale.y,
                offset.z * transform.scale.z,
            );
            let world_offset = transform.rotation * scaled_offset;
            Some((transform.position + world_offset, half_extents))
        })
        .collect();

    for (position, half_extents) in visuals {
        world
            .spawn()
            .with(Renderable { model_id: cube_model_id })
            .with(Transform {
                position,
                rotation: Quaternion::one(),
                scale: half_extents,
            })
            .with(DebugVisual)
            .build();
    }
}
