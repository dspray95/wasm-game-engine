use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    // Player movement
    MoveLeft,
    MoveRight,
    MoveForwards,
    MoveBackwards,
    Fire,
    // Free Cam Movement
    ToggleFreeCamera,
    RotateLeft,
    RotateRight,
    MoveUp,
    MoveDown,
    Pause,
    //Builtins
    ToggleDebugPanel,
}
