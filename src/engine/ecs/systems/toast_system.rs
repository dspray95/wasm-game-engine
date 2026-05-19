use crate::engine::ecs::{
    resources::toasts::ToastQueue, system::SystemContext, world::World,
};

/// Per-frame tick + expire for the global `ToastQueue`.
///
/// Dormant toasts (delay > 0) burn down their delay first; once it hits
/// zero they start ticking their visible lifetime normally. Toasts whose
/// elapsed has passed lifetime are dropped.
pub fn toast_system(world: &mut World, system_context: &mut SystemContext) {
    let delta_time = system_context.delta_time;
    let Some(queue) = world.get_resource_mut::<ToastQueue>() else {
        return;
    };
    for toast in queue.toasts.iter_mut() {
        if toast.delay > 0.0 {
            toast.delay -= delta_time;
            if toast.delay < 0.0 {
                toast.delay = 0.0;
            }
        } else {
            toast.elapsed += delta_time;
        }
    }
    queue.toasts.retain(|toast| toast.elapsed < toast.lifetime);
}
