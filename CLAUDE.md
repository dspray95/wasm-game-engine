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
- **Phase 3** (player/laser via ECS) — in progress
- **Phase 4–5** (city-builder foundation) — planned

## Coding Style

- **No abbreviations in variable names.** Use the full descriptive name: `system_context` not `ctx`, `delta_time` not `dt` (except as a short-lived local after extracting from `system_context.delta_time`), `entity_id` not `id` where the meaning isn't obvious from context.