use crate::{
    engine::ecs::{system::SystemContext, world::World},
    game::{
        input::{actions::Action, world_ext::InputWorldExt},
        resources::{
            game_over_state::{GameOverPhase, GameOverState},
            tutorial_state::TutorialState,
        },
    },
};

pub fn tutorial_system(world: &mut World, system_context: &mut SystemContext) {
    let _ = system_context;

    let phase = world
        .get_resource::<GameOverState>()
        .map(|s| s.phase)
        .unwrap_or(GameOverPhase::Playing);
    if !matches!(phase, GameOverPhase::Playing) {
        return;
    }

    let completed = world
        .get_resource::<TutorialState>()
        .map(|s| s.completed)
        .unwrap_or(true);
    if completed {
        return;
    }

    let input = world.input_state();
    let key_bindings = world.key_bindings();

    let just_left = key_bindings.is_action_just_pressed(&Action::MoveLeft, &input);
    let just_right = key_bindings.is_action_just_pressed(&Action::MoveRight, &input);
    let just_fired = key_bindings.is_action_just_pressed(&Action::Fire, &input);

    if let Some(tutorial) = world.get_resource_mut::<TutorialState>() {
        if just_left {
            tutorial.pressed_left = true;
        }
        if just_right {
            tutorial.pressed_right = true;
        }

        let move_done = tutorial.move_done();
        if move_done && just_fired {
            tutorial.fired = true;
            tutorial.completed = true;
        }
    }
}
