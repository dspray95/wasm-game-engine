# Scene Serialisation

## Current State

Scene layout is declared in `assets/scenes/*.ron` files and loaded via a `ComponentRegistry` that dispatches each component by name to a registered deserializer. Entities are represented as a dict of component name → value:

```ron
(
    entities: [
        {
            "Transform": (position: (x: 24.5, y: -1.0, z: 3.0), ...),
            "Renderable": (model: "starfighter"),
            "Player": (),
        },
    ]
)
```

GPU model loading stays in Rust code (WASM constraint — `include_bytes!` requires compile-time string literals). The scene file references models by name; `load_scene` resolves names to IDs via the `AssetServer`.

---

## The `ron::Value` Problem

The current `SceneDescriptor` uses `HashMap<String, ron::Value>` for each entity's components. `ron::Value` is defined as:

```rust
pub enum Value {
    Bool(bool),
    Char(char),
    Map(Map),
    Number(Number),
    Option(Option<Box<Value>>),
    String(String),
    Seq(Vec<Value>),
    Unit,
}
```

**There is no `Enum` variant.** When RON parses `direction: Up` (a unit enum variant), it's stored as `Value::Unit`, losing the type identity. When `Value::into_rust::<HoverDirection>()` runs, it fails:

```
InvalidValueForType { expected: "enum HoverDirection", found: "a unit value" }
```

Quoting the variant as `"Up"` doesn't help either — it becomes `Value::String("Up")`, and serde's default `deserialize_enum` path for `HoverDirection` doesn't accept a string input when called through Value.

The core issue: **any component containing a Rust enum cannot round-trip through `ron::Value`**. This is a blocker for a scalable scene system — enums are a natural fit for things like `HoverDirection`, `AiState`, `BuildingType`, `Faction`, etc.

---

## Short-Term Workarounds

### (A) Avoid enums in serialisable components
Replace with bool/u8/struct. Fastest fix for a single component but gives up a useful pattern long-term.

```rust
pub struct HoverDirection(pub bool);
```

### (B) Route each enum through a string
Per-enum boilerplate but keeps the variant names readable in RON.

```rust
#[derive(Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum HoverDirection { Up, Down }

impl TryFrom<String> for HoverDirection { /* ... */ }
impl From<HoverDirection> for String { /* ... */ }
```

Both are band-aids. Any new enum component triggers this problem again.

---

## Architectural Fix: Custom `Deserialize` for `SceneDescriptor`

The real fix is to stop using `ron::Value` as an intermediate. Serde's `MapAccess` trait lets us iterate a map's entries and, for each key, call `next_value_seed::<T>()` to deserialize the value *directly* as `T` — bypassing any typed intermediate. This preserves full type fidelity including enums.

### Design

1. `SceneDescriptor` keeps the file-level shape but implements `Deserialize` manually.
2. The deserializer walks each entity, then each component entry, and for each `(name, value)` pair calls into the `ComponentRegistry` using a seed that knows the target type.
3. The registry no longer stores closures taking `ron::Value`; it stores closures that consume a `&mut dyn erased_serde::Deserializer` directly.

### Rough shape

```rust
// Deserializer-based registration
pub struct ComponentRegistry {
    deserializers: HashMap<String, Box<dyn Fn(
        &mut World,
        Entity,
        &mut dyn erased_serde::Deserializer,
    ) -> Result<()>>>,
}

impl ComponentRegistry {
    pub fn register<T: DeserializeOwned + 'static>(&mut self, name: &str) {
        self.deserializers.insert(name.to_string(), Box::new(|world, entity, d| {
            let component: T = erased_serde::deserialize(d)?;
            world.add_component(entity, component);
            Ok(())
        }));
    }
}
```

`SceneDescriptor` implements a custom `Visitor` that takes a reference to the `ComponentRegistry`. The visitor uses `MapAccess` to iterate components and calls `next_value_seed` with a seed that invokes the registry's dispatch function.

Because the registry's closures operate directly on the deserializer, the Value intermediate is gone — and with it, the enum loss.

### Dependencies

- `erased_serde` — lets us pass `&mut dyn Deserializer` through trait objects. Standard crate for this pattern (Bevy uses it under the hood).

### Trade-offs

- **Pro**: full type fidelity. Enums, nested structs, Options, everything works.
- **Pro**: no per-component boilerplate; components just need `#[derive(Deserialize)]`.
- **Pro**: the error messages improve too — failures surface the real expected type, not the lossy Value layer.
- **Con**: ~50-100 lines of custom `Deserialize`/`Visitor`/`DeserializeSeed` code.
- **Con**: the registry can no longer be cloned trivially (trait objects).

### Files to modify

- `src/engine/ecs/component_registry.rs` — swap closure signature to use `erased_serde::Deserializer`
- `src/engine/scene/scene_descriptor.rs` — implement custom `Deserialize` on `SceneDescriptor` that threads the registry through visitors
- `Cargo.toml` — add `erased_serde`
- `load_scene` signature changes: the registry now needs to be passed into the deserializer (likely via `DeserializeSeed`)

### Migration note

The RON file format doesn't change. The string-tag dispatch on component names stays identical. Only the internal dispatch mechanism changes, so all existing scene files keep working.

---

## Recommended Path

1. **Now**: apply workaround (A) to `HoverDirection` (change to struct or bool) to confirm the rest of the scene pipeline is correct end-to-end.
2. **Next**: implement the custom `Deserialize` architecture before adding more enum components. Ideally before adding a second scene file, since that's when the boilerplate cost of workaround (B) would start to hurt.
3. **Later**: once the fix is in, the workaround on `HoverDirection` can be reverted back to a proper enum.

## Verification

- All existing tests pass (`cargo test --lib`)
- `canyon_runner.ron` loads with the player visible, hovering, steering
- A new component with an enum field (e.g. a placeholder `TestEnum { A, B, C }` component) deserialises from a RON scene without error
- Error messages on bad RON point at the actual type mismatch, not "found a unit value"
