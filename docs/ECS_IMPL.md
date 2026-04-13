# ECS Architecture Plan for citizen-engine

## Context

The engine currently manages scene objects via a flat `Vec<Model>` with hardcoded indices in `SceneManager`. This doesn't scale to a city-builder where you need thousands of dynamically spawned/despawned entities (citizens, buildings, vehicles, foliage). A custom ECS will decouple game logic from rendering and provide the foundation for simulation systems.

## Storage Strategy: Sparse Set

**Why sparse set over alternatives:**
- **vs HashMap per component:** HashMap has pointer chasing and poor cache locality. City builders iterate thousands of entities per frame — you'll feel this.
- **vs Archetype-based (Bevy-style):** Best iteration perf but extremely complex to implement from scratch (dynamic column management, archetype graphs, entity migration). Obscures the core ECS concepts you're trying to learn.
- **Sparse set sweet spot:** O(1) insert/remove/lookup, contiguous dense array for iteration, conceptually simple. The tradeoff (cross-component queries need lookups into second set) is acceptable — still O(1) per entity.

## Key Architectural Considerations

1. **Batch processing of homogeneous entities** — sparse set's dense array gives cache-friendly iteration over e.g. all lasers or all terrain chunks in one tight loop.
2. **Mostly static data** — terrain geometry rarely changes per-frame. Use a `RenderDirty` marker component to skip instance buffer rebuilds for entities whose transforms haven't changed.
3. **Dynamic spawning/despawning** — lasers are the immediate test case: spawn on fire, despawn when they expire or travel max distance. Generational entity IDs handle this safely.
4. **Resources as singletons** — `ModelRegistry`, `InputState`, `LaserCooldown` are global, not per-entity. Store in a `Resources` map on `World`, not as components on dummy entities.
5. **Camera stays out of ECS** — it's a singleton with special GPU bind group behavior. Keep it on `EngineState`.

## Core Data Structures

### Entity (generational index)
```rust
pub struct Entity { id: u32, generation: u32 }
pub struct EntityAllocator { generations: Vec<u32>, free_ids: Vec<u32>, next_id: u32 }
```
Generational IDs prevent stale-handle bugs when reusing IDs after despawn.

### SparseSet<T>
```rust
pub struct SparseSet<T> {
    sparse: Vec<Option<usize>>,  // sparse[entity_id] → index into dense/data
    dense: Vec<u32>,              // dense[i] → entity_id
    data: Vec<T>,                 // packed component values
}
```
Insert/remove/get are O(1). Iteration over `data` is cache-friendly.

### World
```rust
pub struct World {
    entities: EntityAllocator,
    components: HashMap<TypeId, Box<dyn Any>>,  // TypeId → SparseSet<T>
    resources: HashMap<TypeId, Box<dyn Any>>,   // global singletons
}
```
No `Component` trait needed — Rust's `TypeId` works on any `'static` type.

### Systems
Simple function pointers run in a fixed order. No complex scheduler needed at this scale.
```rust
pub type System = fn(&mut World, &SystemContext);
pub struct SystemSchedule { systems: Vec<System> }
```
Order: `[input, ai, pathfinding, movement, resource, render_sync]`

### Queries
Start as helper methods on `World` (e.g., `query_two::<Transform, Renderable>()`). For mutable cross-component access, temporarily extract sparse sets from the HashMap to satisfy the borrow checker.

## Renderer Integration

**ModelRegistry** replaces `Vec<Model>` in SceneManager:
- Holds shared GPU mesh data (vertex/index buffers, materials) per model template
- Each `RenderModel` has an instance buffer populated by ECS each frame

**render_sync_system** (runs last):
1. Iterate all `(Renderable, Transform)` entities, group by `model_id`
2. Convert each `Transform` → `InstanceRaw` (reuse existing conversion)
3. Write packed instance data via `queue.write_buffer`
4. Only rebuild buffers for model groups with dirty entities

## File Structure

```
src/engine/ecs/
    mod.rs
    entity.rs           — Entity, EntityAllocator
    sparse_set.rs        — SparseSet<T>
    world.rs             — World (components + resources)
    system.rs            — System, SystemSchedule, SystemContext
    query.rs             — Query helpers (can start as World methods)
```

Components and systems files will be added in later phases as needed.

## Implementation Phases

### Phase 0: Renderer Prep (before ECS)
Improve the renderer before layering ECS on top, so we're not building on a shaky foundation.

**Pre-allocate instance buffers** — files: [mesh.rs](src/engine/model/mesh.rs), [render_state.rs](src/engine/state/render_state.rs)
- Add `max_instances: usize` parameter to `Mesh::new()` — callers specify capacity upfront
- Replace `create_buffer_init()` with `create_buffer()` sized to `max_instances * size_of::<InstanceRaw>()`; if initial instances are provided, write them immediately via `queue.write_buffer`
- Add `instance_count: u32` field to `Mesh` struct; `update_instance_buffer()` sets this, no longer creates a new buffer ever
- Change draw calls in `render_state.rs` from `0..mesh.instances.len() as u32` → `0..mesh.instance_count`
- Note: the current `update_instance_buffer()` is actually broken for growing instance counts (it would write past the end of the buffer) — pre-allocation fixes this too

**Change detection in SparseSet** — design note for Phase 1, not Phase 0 (SparseSet doesn't exist yet):
- When building `SparseSet<T>`, add `dirty: Vec<bool>` and `any_dirty: bool` fields
- `get_mut()` sets `dirty[index] = true` automatically
- Render sync uses `any_dirty` as fast early-out, calls `clear_dirty()` after syncing
- This eliminates any need for a `RenderDirty` marker component

**Frustum culling** — deferred to Phase 2 (render bridge):
- Requires threading the camera VP matrix through to wherever culling runs
- Camera has `build_view_projection_matrix()` but it's not in `RenderContext` yet
- Add to Phase 2 when the render sync system is being built — natural fit

### Phase 1: Core ECS Primitives
Create `entity.rs`, `sparse_set.rs`, `world.rs`, `system.rs`, `mod.rs`. Pure data structures, no GPU dependency. Unit test everything:
- Spawn/despawn with generational recycling
- SparseSet insert/remove/iterate with 1000+ items
- World: register components, add/get/remove, spawn/despawn

### Phase 2: Render Bridge
Connect ECS to the existing wgpu pipeline:
- `Transform` component (maps 1:1 to existing `Instance` struct)
- `Renderable { model_id: usize }` component
- `ModelRegistry` holding shared GPU mesh data + pre-allocated instance buffers
- `render_sync_system` that populates instance buffers from ECS each frame
- Modify `AppState` to hold `World` + `ModelRegistry`, call `SystemSchedule::run_all`
- Modify `RenderState::handle_redraw` to draw from `ModelRegistry`
- Smoke test: starfighter and terrain visible on screen, driven by ECS

### Phase 3: Player & Lasers via ECS
Replace the hardcoded `SceneManager` logic with proper ECS systems:
- `Player` marker component, `Velocity` component
- `Laser { lifetime: f32, max_distance: f32 }` component
- `movement_system`: apply velocity to transform each frame, mark `RenderDirty`
- `laser_system`: spawn laser entity on fire input (with `Velocity`, `Laser`, `Transform`, `Renderable`), despawn when `lifetime` expires — this is the first real test of dynamic entity spawning/despawning
- `player_system`: read input, update player `Velocity` and `Transform`
- `terrain_system`: replaces the chunk-cycling logic in `SceneManager::update`

### Phase 4: City Builder Foundation
Once the canyon runner runs cleanly through ECS, layer in city-builder primitives:
- `Building`, `Zone` components
- `SpatialHash` resource for O(1) proximity queries
- Freeform placement: click → raycast to ground plane → spawn building entity
- `Citizen`, `Velocity`, `AiState` components
- Basic movement system for citizens walking between buildings

### Phase 5: Simulation Systems
- `ResourceProducer`, `ResourceConsumer` components
- `resource_system`: tick production/consumption chains
- Road network via `RoadNode` component

## Critical Files to Modify

- [app_state.rs](src/engine/state/app_state.rs) — hold `World` + `ModelRegistry`, call system schedule instead of `SceneManager::update`
- [render_state.rs](src/engine/state/render_state.rs) — `handle_redraw` accepts `&ModelRegistry` instead of `&[Model]`
- [instance.rs](src/engine/instance.rs) — `Instance`/`InstanceRaw` reused as-is; `Transform` component maps to `Instance`
- [mesh.rs](src/engine/model/mesh.rs) — separate shared GPU data from per-entity instances
- [mod.rs](src/engine/mod.rs) — add `pub mod ecs;`

## Verification

- Phase 1: `cargo test` — all unit tests for entity allocation, sparse set operations, world CRUD
- Phase 2: `cargo run` — starfighter and terrain visible, driven by ECS entities rather than hardcoded `SceneManager` indices
- Phase 3: fire lasers, see them spawn and despawn correctly; player moves via ECS systems
- Phase 4+: visual verification of building placement and citizen movement
