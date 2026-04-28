use cgmath::Vector3;

use crate::engine::{
    ecs::{
        components::{ collider::{ Collider, ColliderShape }, transform::Transform },
        entity::Entity,
        events::collision_event::CollisionEvent,
        system::SystemContext,
        world::World,
    },
    events::events::Events,
};

pub fn collision_system(world: &mut World, _: &mut SystemContext) {
    let snapshot: Vec<(Entity, Vector3<f32>, Collider)> = collect_colliders(world);

    let mut hits: Vec<CollisionEvent> = Vec::new();

    for i in 0..snapshot.len() {
        for j in i + 1..snapshot.len() {
            let (entity_a, pos_a, ref collider_a) = snapshot[i];
            let (entity_b, pos_b, ref collider_b) = snapshot[j];

            if let Some((normal, depth)) = check_collision(pos_a, collider_a, pos_b, collider_b) {
                hits.push(CollisionEvent { a: entity_a, b: entity_b, normal, depth });
            }
        }
    }

    if !hits.is_empty() {
        if let Some(events) = world.get_resource_mut::<Events<CollisionEvent>>() {
            for hit in hits {
                events.send(hit);
            }
        }
    }
}

fn collect_colliders(world: &mut World) -> Vec<(Entity, Vector3<f32>, Collider)> {
    let mut out = Vec::new();
    for (entity_id, collider) in world.iter_component::<Collider>() {
        if let Some(transform) = world.get_component_by_id::<Transform>(entity_id) {
            if let Some(entity) = world.get_entity(entity_id) {
                out.push((entity, transform.position, collider.clone()));
            }
        }
    }
    out
}

fn check_collision(
    pos_a: Vector3<f32>,
    collider_a: &Collider,
    pos_b: Vector3<f32>,
    collider_b: &Collider
) -> Option<(Vector3<f32>, f32)> {
    match (&collider_a.shape, &collider_b.shape) {
        (
            ColliderShape::AABB { half_extents: half_a },
            ColliderShape::AABB { half_extents: half_b },
        ) => {
            aabb_vs_aabb(pos_a, *half_a, pos_b, *half_b)
        }
        // TODO: sphere variants go here
        _ => None,
    }
}

fn aabb_vs_aabb(
    center_a: Vector3<f32>,
    half_a: Vector3<f32>,
    center_b: Vector3<f32>,
    half_b: Vector3<f32>
) -> Option<(Vector3<f32>, f32)> {
    let delta = center_b - center_a;
    let overlap = Vector3::new(
        half_a.x + half_b.x - delta.x.abs(),
        half_a.y + half_b.y - delta.y.abs(),
        half_a.z + half_b.z - delta.z.abs()
    );

    // Any axis with no overlap === no collision
    if overlap.x <= 0.0 || overlap.y <= 0.0 || overlap.z <= 0.0 {
        return None;
    }

    // Smallest overlap axis is the separating direction
    let (axis_normal, depth) = if overlap.x < overlap.y && overlap.x < overlap.z {
        (Vector3::new(delta.x.signum(), 0.0, 0.0), overlap.x)
    } else if overlap.y < overlap.z {
        (Vector3::new(0.0, delta.y.signum(), 0.0), overlap.y)
    } else {
        (Vector3::new(0.0, 0.0, delta.z.signum()), overlap.z)
    };

    Some((axis_normal, depth))
}
