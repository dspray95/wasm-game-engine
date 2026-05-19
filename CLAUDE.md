# CLAUDE.md

Guidance for Claude Code working in this repo.

## Project status

**citizen-engine** is a custom Rust 3D game engine, built ground-up (not a Bevy wrapper). It powers two games:

1. **Canyon Runner** — the engine's first game, currently in polish. Compiles to WASM and runs in-browser.
2. **(unnamed) city builder** — the engine's primary long-term target. Native-only. Work begins after canyon runner ships.

This is no longer a learning project. Decisions should weigh long-term cost: dependency choices, save-format versioning, public API stability, binary size all matter.

## Build & run

```bash
cargo run                                # Native build, runs canyon runner
cargo test --lib                         # Unit tests
cargo test --lib <name>                  # Single test
cargo test --lib -- --nocapture          # With stdout
```

WASM is currently a canyon-runner-only target (the city builder will be native):

```bash
make setup          # Install http-server, first-time setup
make run            # wasm-pack build + serve on :8000
```

Raw WASM build: `RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --out-dir pkg`.

## Architecture

### ECS (`src/engine/ecs/`)

Custom ECS using **sparse sets** for O(1) insert/remove/lookup with cache-friendly iteration over a single component. Joining two components does two sparse lookups per entity — fine at current scale, would be a contiguous scan in archetype-style. See `docs/ECS_IMPL.md` for the full design tradeoff.

- `Entity` — generational ID, prevents stale-handle bugs after despawn
- `World` — owns all component storage and resources
- `SparseSet<T>` — per-component-type storage
- `SystemSchedule` — startup systems once, then frame systems each tick
- `Commands` — deferred mutation buffer on `SystemContext`; queue spawn/despawn, component insert/update/remove, resource update, event send. Flushed after each system returns. **Default way to mutate from a system.**
- `Events<T>` — double-buffered queues, swapped each frame. Producers and consumers don't need to be ordered.
- `ComponentRegistry` — typed RON deserialisation; required for any component appearing in scene files.

**Not yet implemented**: transform hierarchy (`Parent` + composed world transform). Discussed in detail; deferred until needed.

### Rendering (`src/engine/model/`, `src/engine/state/`)

- `ModelRegistry` + `AssetServer` — name → model_id lookup; pre-allocated GPU instance buffers per model
- `render_sync_system` — groups entities by `model_id`, writes instance transforms via `queue.write_buffer`
- `Camera` is an ECS component; `ActiveCamera` resource selects which entity renders
- `engine::ui::projection::world_to_screen` — projects world points to screen pixels for diegetic UI
- Shaders: `src/shader.wgsl` (main), `src/wireframe.wgsl` (debug overlays)
- egui rendered as a separate pass after the scene; games register panels via `UIRegistry`

### Game layout

- `Scene` trait separates engine infrastructure from game content
- Canyon runner entry: `src/game/canyon_runner_world.rs`
- Scene RON: `assets/worlds/canyon_runner.ron`
- Input bindings: `assets/bindings.ron` — systems query named `Action`s, not raw `KeyCode`s

## City builder direction (primary engine target)

The simulation model is **cell-aggregated with hybrid individuals** — not Cities: Skylines-style per-citizen. This shapes architectural decisions:

- The cell grid is a **typed resource** (`Grid<Cell>`), not modelled as entities. Wrong granularity for ECS.
- Buildings, vehicles, named NPCs are ECS entities.
- Two clocks: render at vsync, simulation at fixed timestep (~10 Hz). Save-determinism and frame-rate independence both require this.
- Save layering: cell grid (packed binary) + removal overlay + placed entity records.

### Engine work pending for the city builder

Rough priority order:

1. **Transform hierarchy** — lightweight `Parent { local_offset, local_rotation }` first, full `WorldTransform` split when justified by use cases
2. **Fixed-timestep simulation tick** — separate from frame schedule
3. **`Grid<Cell>` resource pattern** + cell-aware spatial broadphase (replaces current naive O(n²) collision)
4. **LOD / culling** — distance-based mesh swaps, chunk culling, instancing of repeated assets
5. **Async asset loading** — move off `include_bytes!` for non-canyon assets; binary-size cost too high at city scale
6. **Audio** — `kira` or `rodio` for native; absent currently
7. **Pathfinding** — hybrid individuals (vehicles, named NPCs) need it; probably the `pathfinding` crate
8. **Save format with versioning** — bincode + schema version byte at head; migrations inevitable
9. ECS Archetypes - When performance profiling calls for it
10. #[derive] for systems, components, events - an auto-register macro so we don't need to manually register everything. before=system_a type flag for ordering, checks for cyclical systems on build 

Anything outside this list should be flagged before being built.

## Serialisation

- **RON** for human-editable data: building defs, scene archetypes, input bindings, tuning curves
- **bincode** for runtime save state: compact, fast, never edited by hand, always versioned
- Avoid YAML. JSON only for external tool interop.

### Save layering (planned, designed not built)

For city builder. Three layers, in one file:

1. **Authored density mask** — 2D array per cell describing natural vegetation/prop density. Immutable at runtime. Trees placed deterministically from `(seed, cell, idx)` on load.
2. **Removal overlay** — `Vec<(cell, idx)>` recording what the player has destroyed. Grows with activity, not world size.
3. **Placed entity records** — packed bincode of relevant sparse-set dense arrays. Player-built buildings, planted trees, named NPCs, vehicles.

Load = regen from mask → skip removals → instantiate records. File size scales with player activity rather than world area. A replanted tree in a natural cell is a layer-3 entity, not a cancellation of the removal — player-planted carries different semantics.

## Coding style

- **No abbreviations in variable names.** `system_context` not `ctx`, `delta_time` not `dt` (except as a short-lived local), `entity_id` not `id` where context doesn't disambiguate.
- Prefer editing existing files over creating new ones.
- Comments only where the *why* is non-obvious: constraints, invariants, workarounds for specific bugs, surprising behaviour. Skip "what" comments — let identifiers do that.
- Tests inline in their module files (`#[cfg(test)] mod tests`).
- Don't add error handling, fallbacks, or validation for impossible scenarios. Trust internal code; only validate at system boundaries.

## Things to avoid

- **Don't model the cell grid as entities.** Wrong granularity; would create 10⁵+ entities at city scale.
- **Don't add per-citizen entities for the city builder.** The simulation is cell-aggregated.
- **Don't pull in Bevy or other ECS frameworks.** The custom ECS is the engine's value proposition.
- **Don't write planning/decision documents unless explicitly asked.** Use the conversation. The roadmap above is the plan.
- **Don't add features speculatively.** The priority list is the priority list.
