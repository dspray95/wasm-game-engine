use std::sync::atomic::{AtomicU32, Ordering};

use crate::{
    engine::ecs::{resources::toasts::push_hud_toast, system::SystemContext, world::World},
    game::input::{actions::Action, world_ext::InputWorldExt},
};

/// Per-run counter so successive test toasts have distinct text — easier to
/// see stack ordering and overflow behaviour without playing through.
static TOAST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Test-only: pressing F3 (the `TestToast` action) queues a numbered toast
/// so the UI can be validated independently of gameplay. Held key doesn't
/// repeat — we only fire on the press edge.
pub fn debug_toast_system(world: &mut World, system_context: &mut SystemContext) {
    let input = world.input_state();
    let key_bindings = world.key_bindings();

    if !key_bindings.is_action_just_pressed(&Action::TestToast, &input) {
        return;
    }

    let n = TOAST_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    push_hud_toast(system_context.commands, format!("TEST TOAST {n}"));
}
