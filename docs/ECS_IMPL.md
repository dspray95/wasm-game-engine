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
---



  Already done in Phase 3:
  - Starfighter spawned as an ECS entity in canyon_runner_startup (Transform + Renderable) ✓
  - camera_control_system migrated ✓
  - render_sync_system wired up ✓

  Still on the old path:
  - Scene::update() still drives player movement, laser firing, and terrain directly via LaserManager, Starfighter, and TerrainGeneration
  - No Player, Velocity, or Laser components exist yet
  - No player_system, laser_system, or terrain_system exist yet
  - Camera is not accessible from within systems

  ---
  Here's the plan, in order:

  1. Add Velocity component

  src/engine/ecs/components/velocity.rs — pub struct Velocity { pub x: f32, pub y: f32, pub z: f32 }. Export it from components/mod.rs.

  2. Add game-specific components

  Create src/game/components.rs:
  - pub struct Player; — marker, no data
  - pub struct Laser { pub initial_z: f32 } — enough to know when to despawn

  4. Add a FireCooldown resource

  pub struct FireCooldown { pub last_fired: web_time::Instant }. Add it to the world in canyon_runner_startup. This replaces the cooldown currently living inside LaserManager.

  5. Register the laser model in startup

  In canyon_runner_startup, register the laser model in model_registry the same way the starfighter is registered. Store the resulting model_id as a resource (pub struct LaserModelId(pub usize))
  so player_system can reference it when spawning laser entities.

  6. Spawn the player with all components in startup

  When spawning the starfighter entity, chain .with(Player).with(Velocity { x: 0.0, y: 0.0, z: 0.0 }) onto the builder.

  7. Write player_system

  src/game/systems/player_system.rs:
  - Find the entity with Player by iterating iter_component::<Player>()
  - Read InputState resource for A/D input → update Transform x
  - Run hover animation (the Starfighter::animate logic, inlined or kept as a free function)
  - Advance camera z forward by MOVEMENT_SPEED * delta_time via ctx.camera
  - On Space + cooldown met: spawn a laser entity with Transform (at player position), Renderable { model_id }, and Laser { initial_z: player_z }

  8. Write laser_system

  src/game/systems/laser_system.rs:
  - Iterate iter_component::<Laser>() to get entity IDs + initial_z
  - For each, get Transform by entity ID and advance z by (LASER_SPEED + MOVEMENT_SPEED) * delta_time
  - Collect IDs where transform.position.z > initial_z + MAX_LASER_TRAVEL into a Vec first, then despawn them in a second pass — you can't despawn while iterating because that mutably borrows
  World twice

  9. Write terrain_system

  src/game/systems/terrain_system.rs:
  - Store TerrainGeneration and the 3 terrain model IDs as resources on World
  - Read camera z from ctx.camera, call terrain_generation.terrain_update(camera_z)
  - If it returns Some(mesh_data), write the new vertex/index buffers directly via ctx.queue — terrain doesn't go through render_sync_system because it replaces geometry, not instances

  10. Register new systems in setup_ecs

  Add to the schedule in this order: player_system, laser_system, terrain_system, then the existing render_sync_system.

  11. Remove the old code

  Once the systems are running and the game looks right:
  - Delete LaserManager usage and the Starfighter struct usage from the scene
  - Delete move_player() from CanyonRunnerScene
  - Gut Scene::update() — it can become a no-op or be removed from the trait entirely
  - Remove models: Vec<Model> from CanyonRunnerScene (terrain models move to the registry)

  ---
  The two things most likely to trip you up:

  - Two-component queries — World::iter_component<T>() only gives you one component at a time. When you need both Laser and Transform for the same entity, iterate iter_component::<Laser>() for the
   IDs, then call world.get_component_mut::<Transform>(entity) per ID. It's O(1) per lookup, just a bit verbose.
  - Despawn-during-iteration — always collect entity IDs to a Vec<Entity> first, then loop over that vec calling world.despawn(). Trying to do it inline will hit the borrow checker immediately.