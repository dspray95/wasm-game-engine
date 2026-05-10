/// Snapshot of the live entity count, refreshed once per frame from the
/// `EntityAllocator` in `AppState`. UI panels and any other read-only
/// consumer can grab it via `world.get_resource::<EntityCount>()` without
/// needing access to the allocator directly.
pub struct EntityCount(pub usize);
