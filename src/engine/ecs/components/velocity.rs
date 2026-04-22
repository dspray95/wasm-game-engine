use serde::{ Deserialize, Serialize };

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
