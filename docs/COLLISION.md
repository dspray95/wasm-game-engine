# Collision Detection

Design notes and implementation plan for adding collision to the engine. Covers the standard architecture, options at each level, and a phased rollout that fits the current codebase.

## Two-Phase Architecture

Collision splits into two distinct phases — this pattern shows up in every engine from Unity to Box2D to Unreal:

- **Broadphase** — cheap "could these possibly collide?" filter. Returns candidate pairs.
- **Narrowphase** — expensive "do they actually collide and where?" test on each candidate pair.

Why split? Narrowphase is O(1) per pair but expensive (especially mesh-vs-mesh). If you ran narrowphase on every pair, you'd have O(n²) expensive checks. With broadphase you eliminate most pairs cheaply and only do the expensive math on the few that matter.

---

## Broadphase Strategies

Ranked from simplest to most sophisticated:

### Naive O(n²)
Just check every pair. Honest. Works fine up to ~200-500 entities.
```rust
for (a_id, a) in colliders.iter() {
    for (b_id, b) in colliders.iter() {
        if a_id < b_id && bounds_overlap(a, b) { ... }
    }
}
```

### Uniform spatial grid
Divide world into fixed-size cells. Each entity is bucketed by which cell its bounds overlap. Only test pairs that share a cell. Good when:
- Most objects are similar size
- Distribution is roughly uniform
- World has known bounds

For a city builder this is **almost always the right choice** — buildings are roughly grid-sized, the world is bounded, and most queries are local. CLAUDE.md already plans `SpatialHash` for this purpose.

### Sweep and prune (SAP)
Sort entities along an axis, only test pairs whose intervals overlap. Good when most things are static or moving slowly. Maintains state across frames for efficiency.

### Bounding volume hierarchy (BVH)
Tree of nested bounding boxes. Excellent for static geometry (terrain meshes), can be rebuilt or refit for dynamic scenes. Common for raycasting against complex worlds.

### Quadtree / octree
Recursive spatial subdivision. Good for highly non-uniform distributions (e.g. one dense city, lots of empty terrain). More complex than uniform grid; rarely better unless your distribution is genuinely non-uniform.

---

## Narrowphase Shapes

In rough order of cost-vs-precision:

| Shape | Notes | Use for |
|---|---|---|
| **Sphere** | Position + radius. Rotation-invariant. Cheapest. | Projectiles, citizens, simple objects |
| **AABB** | Min + max corners. Rotation breaks it. | Buildings, terrain chunks, anything axis-aligned |
| **OBB** | AABB + rotation. ~5-10x more expensive than AABB. | Vehicles, ships |
| **Capsule** | Sphere swept along a line. | Character controllers (good for stairs, slopes) |
| **Convex hull** | Set of half-planes. Use GJK for collision. | Arbitrary convex shapes |
| **Mesh** | Full triangle soup. Slowest. | Static terrain, usually inside a BVH |

Mixed scenes typically use cheap shapes for moving objects and complex ones for static geometry — e.g. mesh-collider terrain in a BVH, sphere/AABB for everything else.

---

## ECS Integration

A `Collider` component holds a `ColliderShape` enum field. Following the existing convention (see `HoverState` / `HoverDirection`), components are structs and variant data lives in a nested enum:

```rust
pub struct Collider {
    pub shape: ColliderShape,
    // Future metadata fields:
    //   pub is_trigger: bool,   — emit events but don't resolve overlap
    //   pub layer: u32,         — collision layer mask
    //   pub friction: f32,
}

pub enum ColliderShape {
    Aabb { half_extents: Vector3<f32> },
    Sphere { radius: f32 },
    // OBB, Capsule, etc. as you need them
}

pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
    pub normal: Vector3<f32>,  // direction of penetration (a → b)
    pub depth: f32,            // how far they overlap
    // optionally: contact point
}
```

### Why struct + enum, not enum-as-component

Two reasonable options were considered:

1. **`Collider` as a bare enum** — single type, multiple shape variants, dispatch via match.
2. **Separate components per shape** (`AabbCollider`, `SphereCollider`) — most "ECS-pure" but requires separate queries per shape, no shared metadata.

The chosen pattern (struct wrapping enum) matches the existing component convention and leaves room to add shared metadata (`is_trigger`, collision layers, friction) without touching every shape variant. It's also what Bevy's collider integrations use.

The bare-enum approach was rejected because every other component in the codebase is a struct, and consistency wins over micro-optimization at this scale.

The separate-components approach was rejected because broadphase wants to iterate "all colliders" cheaply — separate components mean you need to merge multiple sparse-set queries every frame.

A `collision_system` in `engine_systems` runs late in the frame:

```rust
pub fn collision_system(world: &mut World, _ctx: &mut SystemContext) {
    let pairs = broadphase(world);              // cheap
    for (a, b) in pairs {
        if let Some(contact) = narrowphase(world, a, b) {
            world.events::<CollisionEvent>().send(CollisionEvent { a, b, ... });
        }
    }
}
```

Pair dispatch in narrowphase matches on the inner shape:

```rust
match (&col_a.shape, &col_b.shape) {
    (ColliderShape::Aabb { half_extents: ha }, ColliderShape::Aabb { half_extents: hb }) => {
        aabb_vs_aabb(pos_a, *ha, pos_b, *hb)
    }
    // (ColliderShape::Sphere { ... }, ColliderShape::Sphere { ... }) => sphere_vs_sphere(...),
    // (ColliderShape::Aabb { ... }, ColliderShape::Sphere { ... }) => aabb_vs_sphere(...),
    _ => None,
}
```

Game systems then consume `CollisionEvent`s — damage, particles, audio, score. **This is why event system comes first**: it's the natural communication channel for collision results.

### Static vs dynamic

A useful distinction:
- **Static colliders** — buildings, terrain. Don't move, can be placed in a precomputed structure (BVH or uniform grid built once).
- **Dynamic colliders** — citizens, vehicles, projectiles. Move every frame, broadphase rebuilt or updated incrementally.

The collision pass is then:
1. Dynamic vs static (most pairs)
2. Dynamic vs dynamic (fewer, but more interesting)
3. Static vs static is usually skipped (can't move, can't collide newly)

A `Static` marker component is a cheap way to distinguish.

---

## Phased Rollout

### Stage 1: Minimal AABB + naive O(n²)

- `Collider::Aabb { half_extents: Vec3 }` only
- Naive nested loop in `collision_system`
- Emit `CollisionEvent` per overlapping pair
- Add `collision_system` to `engine_systems` between `velocity_system` and `camera_update_system`

This handles canyon-runner-scale (player, lasers, eventually terrain chunks) and gives a working baseline without spatial structures. Good first ticket — entirely self-contained, ~150 lines.

**Depends on**: event system (must be in place to emit `CollisionEvent`).

### Stage 2: Add Sphere variant + raycasting

- Add `Collider::Sphere { radius: f32 }`
- Mixed-variant pair handling (sphere-vs-aabb, etc.)
- Add a `raycast(world, origin, direction) -> Option<RayHit>` helper

Raycasting uses the same broadphase + narrowphase split. It's needed for:
- Click-to-place in city builder (screen → ray → ground hit)
- Line-of-sight checks
- Laser hit detection (might not need full collision if raycasting suffices)

### Stage 3: Spatial hash for broadphase

- `SpatialHash` resource (uniform grid)
- Rebuild or update incrementally each frame
- Replace naive iteration with grid-cell lookups

Do this when entity count makes naive O(n²) noticeable in profiling. For city-builder scale (thousands of buildings + hundreds of citizens) it's essential. For canyon runner alone, may never be needed.

---

## Gotchas

### Pair ordering and double counting
When iterating, ensure each pair is tested once. The `if a_id < b_id` check is the standard trick.

### Pair-of-self
Don't test entity against itself. Always check `a != b`.

### Static-static collisions
If two static entities overlap, do you care? Usually no — building overlap is a placement rule, not a per-frame event. Skip with a `Static` marker.

### Collision response is downstream
The collision *system* just detects. Whether you push entities apart, apply damage, play audio — that's all consumer responsibility, downstream of the event. Keep the collision system pure detection.

### Frame ordering
Collision system runs **after** `velocity_system` (so positions are current) but **before** `render_sync_system` (so rendering reflects post-collision state if you do response). The engine_systems order becomes:

```
[velocity, collision, camera_update, render_sync]
```

### Continuous vs discrete collision
Discrete collision tests once per frame at the current position — misses fast-moving small objects (laser through a thin wall). Continuous collision sweeps along the velocity vector but is more complex. Lasers especially might need a raycast variant rather than collider+collider testing.

---

## Tickets

1. **AABB collider + naive collision_system** — depends on event system. First cut.
2. **Sphere collider + raycasting** — adds shape variety plus the picking primitive city builder needs.
3. **Spatial hash broadphase** — defer until profiling demands it.

---

## Reference: Worked Examples

### Canyon runner needs
- **Laser → terrain**: probably raycast (Stage 2) rather than full collision — laser is a fast-moving thin object, ideal for ray semantics.
- **Laser → enemy**: AABB or sphere collision (Stage 1).
- **Player → terrain**: AABB or capsule, with response (push player out of overlap).

### City builder needs
- **Click → ground**: raycast through the camera (Stage 2). Foundation for placement, selection, hover.
- **Building placement**: AABB-vs-AABB at would-be-spawn-position. If overlapping any existing static, deny placement. Pure broadphase query.
- **Citizen → building**: AABB or sphere, used for "am I at my destination" checks rather than physics response.
- **Vehicle → vehicle**: needs OBB if you want correct rotation handling, otherwise sphere is acceptable.