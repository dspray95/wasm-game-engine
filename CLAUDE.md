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

The ECS uses **sparse sets** for O(1) insert/remove/lookup with cache-friendly dense iteration. Key types:

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
- `Camera` — singleton (not in ECS) with its own bind group; holds view/projection matrices

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
- **Phase 4** (engine foundations) — planned:
  - **RON asset loading** — `ModelDescriptor` RON schema, `ModelLoader` parses via `include_str!` at compile time, calls `load_mesh_from_arrays`, registers in `ModelRegistry`. Moves all vertex data out of Rust code into `assets/*.ron` files. WASM-safe (no runtime file I/O).
  - **Camera into ECS** — Camera becomes a component on an entity rather than a singleton resource. Add `CameraFollow` component to express tracking relationships. Enables `velocity_system` to move the camera naturally and supports multiple cameras (minimap, reflections) later.
  - **egui UI** — integrate `egui` with its `wgpu` backend for in-game UI. Immediate-mode, well-maintained, good fit for debug panels and eventual city-builder HUD. Renders as a separate pass after the main scene.
  - **Input action layer** — mappings loaded from `assets/bindings.ron` at startup; `InputState` exposes `is_action_pressed("strafe_left")` rather than raw `KeyCode`s. Systems declare intent via action names, letting users rebind keys and decoupling game logic from winit.
  - **Transform hierarchy** — `Parent(Entity)` component plus a `hierarchy_system` that composes child local transforms with parent world transforms before `render_sync_system` runs. Enables attaching props to the ship, wheels to vehicles, signage to buildings.
  - **Collision detection** — `Collider` component (AABB/sphere variants) and a `collision_system` that detects overlaps and pushes `CollisionEvent`s to the event system. Broadphase starts naive (O(n²)); swap in a spatial grid once entity counts grow.
  - **Debug overlay** — egui panel showing FPS, frame time, entity count, active resources, and per-system timings. Togglable via a debug action; essential for diagnosing the city-builder's perf profile later.
  - **Event system** — generic `Events<T>` resource with double-buffered queues so producers and consumers can live in different systems without ordering constraints. Replaces direct cross-system coupling (e.g. collisions → damage, input → UI).
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

### Input Bindings (planned)
Input action mappings will be stored in `assets/bindings.ron` and loaded at startup into `InputState`. Systems query named actions (`"strafe_left"`, `"fire"`) rather than raw `KeyCode`s directly. Use `include_str!("../assets/bindings.ron")` for WASM compatibility (no filesystem access in browser).

## Coding Style

- **No abbreviations in variable names.** Use the full descriptive name: `system_context` not `ctx`, `delta_time` not `dt` (except as a short-lived local after extracting from `system_context.delta_time`), `entity_id` not `id` where the meaning isn't obvious from context.