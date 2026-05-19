use crate::engine::ecs::{
    resources::toasts::ToastQueue, system::SystemContext, world::World,
};

/// Per-frame tick + expire for the global `ToastQueue`. Increments every
/// toast's `elapsed`, drops toasts that have lived past their lifetime.
/// Rendering is the panel's job; this system is purely state.
pub fn toast_system(world: &mut World, system_context: &mut SystemContext) {
    let delta_time = system_context.delta_time;
    let Some(queue) = world.get_resource_mut::<ToastQueue>() else {
        return;
    };
    for toast in queue.toasts.iter_mut() {
        toast.elapsed += delta_time;
    }
    queue.toasts.retain(|toast| toast.elapsed < toast.lifetime);
}
