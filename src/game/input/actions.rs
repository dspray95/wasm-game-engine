use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    // Player movement
    ModeLeft,
    MoveRight, 
    MoveForwards, 
    MoveBackwards,
    Fire,
    // Free Cam Movement
    RotateLeft,
    RotateRight,
    MoveUp,
    MoveDown,
}