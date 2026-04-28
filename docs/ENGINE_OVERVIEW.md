# Engine Overview

A jumping-off point for understanding how the engine fits together. Covers the core engine modules under `src/engine/` only — game-specific code under `src/game/` lives on top of these foundations and is intentionally out of scope here.

## Top-Level Layout

```
src/engine/
├── app.rs              — winit application handler, window + event routing
├── state/              — application state, GPU state, render state
├── ecs/                — custom Entity Component System
├── assets/             — asset server, RON loader
├── scene/              — scene trait + RON scene loader
├── input/              — input state, key binding descriptors
├── ui/                 — egui integration + UI panel registry
├── model/              — model loading, mesh, material, vertex, draw
├── instance.rs         — instance buffer types for GPU instancing
├── light.rs            — global light uniform
├── texture.rs          — texture loading + creation
├── render_pipeline.rs  — wgpu render pipeline construction
└── fps_counter.rs      — frame-rate counter (lives as a World resource)
```

## How It Fits Together

The engine has four major subsystems that talk to each other through `World`:

```
                ┌─────────────────────────────────┐
                │            App                  │
                │  (winit ApplicationHandler)     │
                └───────────────┬─────────────────┘
                                │
                        events  │  redraw
                                ▼
                ┌─────────────────────────────────┐
                │          AppState               │
                │  owns: window, engine_state,    │
                │         render_state, world,    │
                │         egui_state, ui_registry │
                └───┬──────────┬──────────┬───────┘
                    │          │          │
                    ▼          ▼          ▼
              EngineState  RenderState  World
              (GPU init)   (passes)     (ECS state)
                                            │
                                            ▼
                                  ┌─────────────────┐
                                  │  SystemSchedule │
                                  │  startup → game │
                                  │  → engine sys.  │
                                  └─────────────────┘
```

`World` is the single source of truth for game state. Everything else is plumbing.

---

## ECS

A custom sparse-set ECS, intentionally hand-built (not Bevy or specs) as a learning exercise and to keep dependencies minimal. See `docs/ECS_IMPL.md` for the original design rationale.

### Core types

- **`Entity { id: u32, generation: u32 }`** ([entity.rs](../src/engine/ecs/entity.rs)) — generational index. Generations prevent stale-handle bugs after despawn.
- **`SparseSet<T>`** ([sparse_set.rs](../src/engine/ecs/sparse_set.rs)) — backing storage per component type. `sparse[entity_id] → dense_index → data[dense_index]`. O(1) insert/remove/lookup, contiguous dense iteration.
- **`World`** ([world.rs](../src/engine/ecs/world.rs)) — owns all component storages and resources. Components keyed by `TypeId`, stored as `Box<dyn ComponentStorage>` and downcast to `SparseSet<T>` when accessed.

### Adding components

```rust
world.spawn().with(Transform::new()).with(Velocity { ... }).build();
// or
let entity = world.spawn_entity_only();
world.add_component(entity, Transform::new());
```

### Querying

`Query` ([query.rs](../src/engine/ecs/query.rs)) provides typed iteration over entities matching a component set:
```rust
for (transform, velocity) in world.query_iter::<(&mut Transform, &Velocity)>() {
    transform.position += velocity.0 * dt;
}
```

### Resources

Type-keyed singletons on `World`. Used for things that are not per-entity:
- `InputState`, `FpsCounter`, `ActiveCamera(Entity)`, `SurfaceDimensions`, `CameraBindGroupLayout`, etc.

```rust
world.add_resource(FpsCounter::new());
let fps = world.get_resource::<FpsCounter>().unwrap();
```

### Systems

A `System` is a function pointer:
```rust
pub type System = fn(&mut World, &mut SystemContext);
```

`SystemContext` carries per-frame inputs that aren't on `World`:
- `delta_time: f32`
- Optional `&wgpu::Device`, `&wgpu::Queue`, `&mut AssetServer` (only for systems that need GPU access)

`SystemSchedule` ([system.rs](../src/engine/ecs/system.rs)) runs systems in a fixed order, split into three buckets:

1. **`startup_systems`** — run once on first tick (scene initialization)
2. **`game_systems`** — game-side logic (added by `Scene::setup_ecs`)
3. **`engine_systems`** — fixed engine systems (currently `velocity_system`, `camera_update_system`, `render_sync_system`, in that order)

Engine systems always run last so they pick up all logic mutations from game systems before pushing to the GPU.

### Built-in engine systems

- **`velocity_system`** ([systems/velocity_system.rs](../src/engine/ecs/systems/velocity_system.rs)) — applies `Velocity` to `Transform` each frame.
- **`camera_update_system`** ([systems/camera_update_system.rs](../src/engine/ecs/systems/camera_update_system.rs)) — reads `ActiveCamera` entity's `Transform`, updates the camera's view-projection matrix, uploads to GPU.
- **`render_sync_system`** ([systems/render_sync_system.rs](../src/engine/ecs/systems/render_sync_system.rs)) — groups all `(Renderable, Transform)` entities by `model_id`, builds instance buffers, uploads via `queue.write_buffer`. The bridge between ECS and rendering.

### Component registry

`ComponentRegistry` ([component_registry.rs](../src/engine/ecs/component_registry.rs)) — maps string names to type-erased deserializer closures using `erased_serde`. Used by the scene loader to spawn entities from RON files without a central enum. See `docs/SCENE_SERIALISATION.md` for the architectural details.

### Engine resources catalog

A reference list of resources installed into `World` by the engine itself (not by game scenes). Game scenes layer their own resources on top — see `src/game/resources/` for those. This catalog is the canonical "what's in the world" map for engine-side state.

**Conventions for adding a resource**:
- Must be a singleton (one instance per world)
- Must be cross-cutting (multiple systems read or write)
- Pure data preferred; thin handles acceptable; no GPU-owning state with complex lifecycles (those live on `AppState`)
- If only one system uses it, make it a local in that system instead

| Resource | Type | Where written | Where read |
|---|---|---|---|
| `InputState` | engine input | `AppState::handle_keyboard_input`, `clear_transient` per frame | game systems, UI panels |
| `FpsCounter` | engine timing | `AppState::update` (each frame) | UI panels (debug) |
| `ActiveCamera(Entity)` | ECS pointer | scene startup (`world.create_active_camera`) | `camera_update_system`, render path, resize handler |
| `SurfaceDimensions` | engine state | `AppState::install_window_state`, `handle_resized` | systems needing aspect ratio (camera projection on resize) |
| `CameraBindGroupLayout` | GPU handle | `AppState::install_window_state` (forwarded from `EngineState::new`) | scene startup when spawning camera entities |
| `EventRegistry` | engine infrastructure | `AppState::install_window_state`, `register_event::<T>` calls | `event_swap_system` |
| `Events<T>` | engine infrastructure (one per event type) | producer systems via `events_mut().send(...)` | consumer systems via `events().read()` |

**Game-specific resources** (out of scope for this engine doc) live in `src/game/resources/` and are listed here only as examples of the pattern: `Bindings<Action>`, `MovePlayer(bool)`, `FreeCameraEnabled(bool)`, `ShowDebugPanel(bool)`, `TerrainGeneration`, `TerrainModelIds`, `LaserModelId`, `LaserManager`.

**Resources NOT in `World`** (intentional — these have natural owners on `AppState` instead):
- `EngineState`, `RenderState`, `EguiState`, `UIRegistry`, `SystemSchedule` — owned by `AppState` because they're consumed by the main loop, not by systems
- `AssetServer` — owned by `AppState`, passed into systems via `SystemContext` because it needs `&mut` access from `render_sync_system`

---

## Rendering

The rendering layer is wgpu-backed and built around instanced draws.

### `EngineState` ([state/engine_state.rs](../src/engine/state/engine_state.rs))

Owns GPU primitives initialized once at startup:
- `wgpu::Device`, `wgpu::Queue`, `wgpu::Surface`
- Render pipelines (main + wireframe)
- Depth texture, MSAA color/depth textures
- Light uniform + bind group

Constructed in `App::resumed` once the window exists. Returns `(EngineState, wgpu::BindGroupLayout)` — the layout is needed to spawn camera entities later.

### `RenderState` ([state/render_state.rs](../src/engine/state/render_state.rs))

Per-frame rendering. Currently very thin — owns just the clear color. `handle_redraw` does:

1. Acquire surface texture
2. Clear pass (always)
3. 3D scene pass (if `ActiveCamera` exists):
   - Wireframes behind transparent (LoadOp::Load)
   - Opaque geometry (LoadOp::Load)
   - Wireframes on top
4. egui pass (composites UI on top of scene)
5. `queue.submit` and `surface.present`

All passes share one command encoder; only one submit per frame.

### `Model` + `ModelRegistry` ([model/](../src/engine/model/))

`Model` holds shared GPU mesh data plus a pre-allocated instance buffer per mesh. `ModelRegistry` is a `Vec<Model>` keyed by `usize` ids. Each `Model` is uploaded once at load time; per-frame instance data is written via `queue.write_buffer` rather than allocating new buffers.

### Instancing

Each entity with a `Renderable` component (carrying a `model_id`) and a `Transform` component contributes an `InstanceRaw` to its model's instance buffer. `render_sync_system` groups by `model_id` and writes packed instance data each frame. One draw call per model, regardless of entity count.

### Render contexts

`state/context.rs` defines small grouped-borrow types passed around by render code:
- `GpuContext { device, queue }` — minimal, for asset loading
- `RenderContext { ... }` — full set for `RenderState::handle_redraw`
- `EguiContext { state, full_output, window }` — egui state needed for the UI pass

---

## Application Loop

### `App` ([app.rs](../src/engine/app.rs))

Implements `winit::ApplicationHandler`. Two responsibilities:

1. **`resumed`** — first window creation. Constructs the window, surface, `EngineState`, `RenderState`, and the initial `Scene`, then calls `AppState::install_window_state`.
2. **`window_event`** — dispatches winit events. Forwards everything to egui first (via `EguiState::on_window_event`), gates input-affecting events on egui's `consumed` flag, and routes the rest to AppState.

Cross-platform-aware via `#[cfg(target_arch = "wasm32")]` blocks (e.g. async GPU init for WASM, debounced canvas resize). The native path is the primary target now.

### `AppState` ([state/app_state.rs](../src/engine/state/app_state.rs))

Holds runtime state:
- `instance: wgpu::Instance` (created early so it's available for surface creation)
- `Option<EngineState>`, `Option<RenderState>`, `Option<Window>` (filled in once the window is created)
- `Option<World>`, `Option<AssetServer>`, `Option<SystemSchedule>`, `Option<UIRegistry>`, `Option<EguiState>`

The `Option`s are because `AppState::new` runs before `winit` has produced a window. Everything that needs GPU/window context gets installed in `install_window_state`.

`handle_redraw_requested` is the per-frame entry point — see "Frame Lifecycle" below.

---

## Input

### `InputState` ([input/input_state.rs](../src/engine/input/input_state.rs))

A World resource. Three sets:
- `pressed: HashSet<KeyCode>` — currently held keys
- `just_pressed: HashSet<KeyCode>` — pressed this frame (one-frame edge)
- `just_released: HashSet<KeyCode>` — released this frame (one-frame edge)

Plus `active_modifiers` (ctrl/shift/alt) tracked alongside.

`AppState::handle_keyboard_input` is the sole writer (called from `App::window_event`). All consumers — game systems, UI panels — read from `World`. After all consumers run, `clear_transient()` wipes the `just_*` sets.

### Critical timing detail

`clear_transient()` runs **after both ECS systems and UI panels** have read input for the frame. It lives at the top level of `handle_redraw_requested`, immediately after the egui run closure. If you add a new input consumer that runs after egui (e.g. modal dialogs), it must fit before this clear.

### `Bindings` (game-side)

`Bindings<Action>` is a game-side abstraction (under `src/game/input/`) that maps named actions (`Action::Fire`, etc.) to `KeyCode + Modifiers`. Loaded from `assets/bindings.ron`. The engine itself doesn't know about actions — it provides `InputState`, the game layers actions on top.

---

## Assets

### `AssetServer` ([assets/server.rs](../src/engine/assets/server.rs))

Wraps `ModelRegistry` with a name → id `HashMap`. Models are loaded once, registered with a string name, and looked up later by name. Lives as an optional field on `AppState` (not a World resource — it needs `&mut` access from the render sync system, which gets it via `SystemContext`).

```rust
asset_server.register_model("starfighter", model);
let id = asset_server.get_model_id("starfighter");
```

GPU model data (vertex/index/instance buffers) is constructed at registration time. Cannot be data-driven from RON because WASM has no filesystem — model bytes come from `include_bytes!`.

### RON loader

`assets/ron_loader.rs` provides `parse_ron_or_log`, a small helper that parses any RON file into a typed descriptor, logging errors instead of panicking. Used for `bindings.ron` and similar config files.

---

## Scenes

### `Scene` trait ([scene/scene.rs](../src/engine/scene/scene.rs))

The hook for game code to register systems and UI panels:

```rust
pub trait Scene {
    fn setup_ecs(&self, schedule: &mut SystemSchedule) {}
    fn setup_ui(&self, ui_registry: &mut UIRegistry) {}
}
```

Each game scene (e.g. `CanyonRunnerScene`) implements this. The engine calls both methods during `install_window_state`.

### Scene descriptor / RON loading

`scene/scene_descriptor.rs` implements the custom `Deserialize` chain (`SceneDescriptorSeed → EntityListSeed → EntitySeed → ComponentSeed`) that drives `World` directly from a RON file. Each component name is dispatched through the `ComponentRegistry` to its registered deserializer. `Renderable` is special-cased — model name string is resolved to `model_id` via `AssetServer` at load time.

See `docs/SCENE_SERIALISATION.md` for full details on the dispatch architecture and the rationale behind it.

---

## UI (egui)

### `EguiState` ([ui/egui_state.rs](../src/engine/ui/egui_state.rs))

Owns the three egui pieces:
- `egui::Context` — the immediate-mode UI state
- `egui_winit::State` — bridges winit events to egui
- `egui_wgpu::Renderer` — tessellates and draws egui output via wgpu

Three methods:
- `on_window_event(window, event)` — forward winit event to egui, returns whether egui consumed it
- `run(window, build_ui)` — runs one egui frame, returns `FullOutput` for rendering
- `render(device, queue, encoder, surface_view, window, full_output)` — issues the egui render pass on an existing command encoder (LoadOp::Load, sample_count 1, no depth)

### `UIRegistry` ([ui/ui_registry.rs](../src/engine/ui/ui_registry.rs))

Mirrors `SystemSchedule` for UI panels. A panel is a function:

```rust
pub type UIPanel = fn(&egui::Context, &mut World);
```

Game code registers panels in `Scene::setup_ui`. The registry's `draw_all` is called inside the egui run closure each frame.

Panel state lives as World resources (e.g. `ShowDebugPanel(bool)`), keeping panels stateless functions and matching the rest of the engine's architecture.

### Built-in panels

- **`debug_panel`** ([ui/built_in/debug_panel.rs](../src/engine/ui/built_in/debug_panel.rs)) — toggleable debug overlay showing FPS and entity count. Toggle key bound via the game's `Action::ToggleDebugPanel`.

---

## Frame Lifecycle

What happens during one `handle_redraw_requested`, in order:

```
1. device.poll(Wait)                  — wait for GPU to be ready
2. update():
   2a. FpsCounter.update()            — bump counter
   2b. Compute delta_time
   2c. SystemSchedule.run_all:
       - startup_systems (first frame only)
       - game_systems (player, hover, terrain, laser, ...)
       - engine_systems (velocity → camera_update → render_sync)
3. egui_state.run(...):
   - ui_registry.draw_all → each registered UIPanel
4. InputState.clear_transient()       — wipe just_pressed/released after consumers
5. render_state.handle_redraw:
   - clear pass
   - 3D scene pass (if ActiveCamera)
   - egui pass
   - queue.submit + surface.present
6. window.request_redraw()            — schedule next frame
```

Keyboard events arrive before redraw via `App::window_event`, recorded into `InputState` for the next redraw to consume.

---

## Things Worth Knowing

- **No archetype storage.** Sparse sets mean cross-component queries do two lookups per entity (renderable → entity_id → transform). Acceptable at current scale; revisit if profiling demands it. See `render_sync_system` comments for the upgrade path.
- **`render_sync_system` allocates a fresh HashMap each frame.** Cheap at small entity counts; replace with a persistent `Vec<Vec<InstanceRaw>>` resource that's `clear()`-ed each frame when entity counts grow.
- **Camera is an entity, not a singleton.** `ActiveCamera(Entity)` resource points to the current one. Renderer skips the 3D pass entirely if no `ActiveCamera` exists (e.g. menu scenes).
- **One render submit per frame.** Scene + egui share a command encoder.
- **Native and WASM both target the same code paths.** The city-builder direction is native-only, so WASM will likely be retired from this codebase.

## Where To Start When Adding Features

| Want to add… | Touch… |
|---|---|
| New gameplay system | `src/game/systems/`, register in `Scene::setup_ecs` |
| New component type | `src/game/components/`, derive `Serialize/Deserialize` if it should appear in scene RON, register in `ComponentRegistry` |
| New UI panel | `src/game/ui/panels/` (or `engine/ui/built_in/` if engine-level), register in `Scene::setup_ui` |
| New input action | Add variant to `Action` enum, add binding in `assets/bindings.ron` |
| New world resource | `world.add_resource(...)` somewhere in scene startup |
| New rendering capability | `src/engine/state/render_state.rs` for pass-level changes; `src/engine/render_pipeline.rs` for new pipelines |
