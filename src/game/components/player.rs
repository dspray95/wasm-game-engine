use serde::{Deserialize, Serialize};

const Z_MOVEMENT_SPEED: f32 = 10.0;
const STRAFE_SPEED: f32 = 4.0;

fn default_z_movement_speed() -> f32 { Z_MOVEMENT_SPEED }
fn default_strafe_speed() -> f32 { STRAFE_SPEED }
fn default_move_player() -> bool { true }

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Player {
    #[serde(default = "default_z_movement_speed")]
    pub z_movement_speed: f32,
    #[serde(default = "default_strafe_speed")]
    pub strafe_speed: f32,
    #[serde(default = "default_move_player")]
    pub move_player: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            z_movement_speed: Z_MOVEMENT_SPEED,
            strafe_speed: STRAFE_SPEED,
            move_player: true,
        }
    }
}
