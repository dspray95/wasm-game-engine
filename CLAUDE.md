# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

This is a Rust project that compiles to WebAssembly via wasm-pack.

```bash
make setup    # Install http-server and build WASM (first time)
make build    # Compile to WebAssembly → pkg/
make serve    # Serve on http://localhost:8000
make run      # build + serve
```

The raw wasm-pack command (used by Makefile):
```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --out-dir pkg
```

Note: Native `cargo run` is currently non-functional. The game runs in-browser only.

## Tests

Tests live inline in their module files. Run with:

```bash
cargo test --lib                      # All unit tests
cargo test --lib <test_name>          # Single test by name
cargo test --lib -- --nocapture       # With stdout
```

## Architecture

**citizen-engine** is a Rust/WASM 3D game engine being built toward a city-builder simulator. It currently runs a "canyon runner" demo. The engine is built around a custom ECS (Entity Component System) — this is an intentional learning project, not a wrapper around Bevy or similar.

### ECS Core (`src/engine/ecs/`)

The ECS uses **sparse sets** for O(1) insert/remove/lookup with cache-friendly iteration over a single component. Joining two components (`Renderable` + `Transform`) does two sparse lookups per entity — fine at current scale, would be a contiguous scan in an archetype ECS. See `docs/ECS_IMPL.md` for the full tradeoff. Key types:

- `Entity` — generational ID (index + generation) to prevent stale-handle bugs after despawn
- `World` — owns all component storage and resources; is the single source of truth
- `SparseSet<T>` — backing storage per component type
- `SystemSchedule` — runs startup systems once, then frame systems each tick (input → logic → render_sync)
- `Resources` — type-erased map on `World` (e.g., `InputState`)

### Rendering (`src/engine/model/`, `src/engine/state/`)

GPU rendering is wgpu-based with instanced draw calls:

- `ModelRegistry` — central registry of models, each with pre-allocated GPU instance buffers
- `render_sync_system` — ECS frame system that groups entities by `model_id`, writes instance transforms to GPU buffers via `queue.write_buffer`
- `EngineState` — wgpu device/queue/surface setup
- `RenderState` — manages render passes, depth texture, draw calls
- `Camera` — ECS component; the `ActiveCamera` resource holds the entity ID of the camera currently rendering. Camera entities carry a `Transform` for position and a `Camera` component for view/projection state and the wgpu bind group.

Shaders are WGSL: `src/shader.wgsl` (main), `src/wireframe.wgsl`.

### Scene Abstraction (`src/engine/scene/`, `src/game/`)

`Scene` trait separates game content from engine infrastructure. `CanyonRunnerScene` is the current implementation. Game-specific systems (player movement, laser spawning, terrain cycling) live in `src/game/systems/`.

### ECS ↔ Rendering Bridge

The key data flow:

```
InputState (resource) → game systems → ECS transforms → render_sync_system → GPU instance buffers → wgpu draw
```

Each entity with a `Renderable` component (carrying a `model_id`) and a `Transform` component is picked up by `render_sync_system` and batched into the appropriate model's instance buffer.

### ECS Roadmap

See `docs/ECS_IMPL.md` for the full design document. Current status:
- **Phase 1** (ECS core) ✓
- **Phase 2** (render bridge) ✓
- **Phase 3** (player/laser via ECS) ✓
- **Phase 4** (engine foundations) — substantially complete; remaining items called out below:
  - **OBJ asset loading + AssetServer** ✓ — `load_model_from_obj_bytes` parses OBJ/MTL via `include_bytes!` at compile time. `AssetServer` wraps `ModelRegistry` with a name→ID `HashMap`, so systems look up models via `asset_server.get_model_id("starfighter")` instead of holding wrapper resources.
  - **Camera into ECS** ✓ — `Camera` is a component; `ActiveCamera` resource selects which entity's camera renders. `CameraFollow` for tracking relationships and support for multiple cameras (minimap, reflections) remain future work.
  - **Scene serialisation (RON)** ✓ — `assets/worlds/canyon_runner.ron` declares models + entity archetypes; `ComponentRegistry` dispatches tagged enums to per-type deserialisers via `#[derive(Serialize, Deserialize)]` on all components. Replaces hand-coded startup functions. Shares its registry with the planned bincode save path.
  - **egui UI** ✓ — integrated with the wgpu backend; renders as a separate pass after the main scene. `UIRegistry` lets games register additional panels (e.g. `score_counter`, `debug_panel`).
  - **Input action layer** ✓ — `assets/bindings.ron` loaded at startup; `InputState`/`KeyBindings` expose `is_action_pressed(&Action::Fire, &input)` rather than raw `KeyCode`s. Systems declare intent via the `Action` enum, decoupled from winit.
  - **Collision detection** ✓ — `Collider` component (AABB; sphere variant declared but not yet implemented) and `collision_system` push `CollisionEvent`s into the event system. Broadphase is currently naive O(n²); swap in a spatial grid once entity counts grow.
  - **Debug overlay** ✓ — egui panel showing FPS, entity count, and game-side difficulty curves (player speed, enemy spawn interval, laser cooldown). Togglable via `F1`. `collider_debug_system` (`F2`) overlays wireframe AABBs.
  - **Event system** ✓ — generic `Events<T>` resource with double-buffered queues swapped each frame so producers and consumers live in different systems without ordering constraints. Used for `CollisionEvent`, `EnemyKilledEvent`, `PlayerDiedEvent`, `LaserFiredEvent`, `ScoreEvent`.
  - **Commands** ✓ (new since the original roadmap) — `Commands` deferred-mutation buffer on `SystemContext`; systems queue spawn/despawn, component insert/update/remove, resource update, and event send operations; the schedule flushes after each system returns. Resolves overlapping `&mut World` borrow conflicts and is now the default way systems mutate world state.
  - **Transform hierarchy** — *not yet implemented*. `Parent(Entity)` component plus a `hierarchy_system` that composes child local transforms with parent world transforms before `render_sync_system` runs. Needed for attaching props to ships, wheels to vehicles, signage to buildings.
- **Phase 5** (city-builder foundation) — planned

## Serialisation

### Formats
- **RON** (Rusty Object Notation) is the preferred format for human-editable data — building templates, terrain configs, entity archetypes, input bindings. It understands Rust types natively (structs, enums, Options) and supports comments.
- **bincode** is the preferred format for runtime save files (city saves, game state). Compact and fast; players never read it directly.
- Avoid YAML (indentation-sensitive, subtle type coercion bugs). JSON is acceptable for interop with external tools only.

### City Builder Save System (planned)
The city-builder will need full world serialisation — every entity and its components at save time. The intended approach:

1. Derive `serde::Serialize`/`Deserialize` on all components
2. Build a **component registry** on `World` — a `HashMap<TypeId, Box<dyn SerialiseStorage>>` that maps each storage to a type-erased serialise/deserialise function
3. At save time: iterate all entities, serialise each component storage → write to bincode
4. At load time: deserialise each storage → respawn entities with their components
5. Authored data (building definitions, terrain configs) lives in RON files under `assets/`

The hard part is the type-erased component registry — `World` is currently unaware of which types it stores beyond `TypeId`. A proc macro or explicit registration step will be needed. See Bevy's `Reflect` trait for prior art.

### Save File Layering (planned)
Saving every tree/rock/prop as a full ECS entity does not scale — a large map has hundreds of thousands of them. Cities: Skylines-style games handle this with packed binary arrays, but we can go further by splitting persistent state into three layers:

1. **Authored density mask** — a 2D array (e.g. 1 byte per cell) describing natural vegetation/prop density. Immutable at runtime, small, loads instantly. Trees are placed deterministically from `(seed, cell_coords, tree_index)` on load.
2. **Removal overlay** — `Vec<(cell_x, cell_y, tree_idx)>` recording which procedural trees the player removed. Grows with player destruction, not world size. On load, procedural regen runs then skips these positions.
3. **Placed entities** — full packed records for everything the player built (buildings, roads, planted trees, citizens, vehicles). Serialised as bincode of the relevant sparse-set dense arrays.

Save file = mask + overlay + packed records. Load = regen from mask → skip removals → instantiate records. File size scales with *player activity* rather than world size, and the player-facing fiction of "every tree is persistent" holds for anything they interacted with. Pure background scenery is allowed to reshuffle imperceptibly between sessions.

A replanted tree in a natural cell becomes a new placed entity (layer 3), not a cancellation of the removal — player-planted trees carry different semantics (species choice, "planted by player" flag for gameplay).

### Input Bindings (planned)
Input action mappings will be stored in `assets/bindings.ron` and loaded at startup into `InputState`. Systems query named actions (`"strafe_left"`, `"fire"`) rather than raw `KeyCode`s directly. Use `include_str!("../assets/bindings.ron")` for WASM compatibility (no filesystem access in browser).

## Coding Style

- **No abbreviations in variable names.** Use the full descriptive name: `system_context` not `ctx`, `delta_time` not `dt` (except as a short-lived local after extracting from `system_context.delta_time`), `entity_id` not `id` where the meaning isn't obvious from context.