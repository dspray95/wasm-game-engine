use serde::{ Deserialize, Serialize };

#[derive(PartialEq, Serialize, Deserialize)]
pub struct Laser {
    pub initial_z: f32,
}
