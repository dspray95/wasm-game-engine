use serde::{ Serialize, Deserialize };

#[derive(PartialEq, Serialize, Deserialize)]
pub enum HoverDirection {
    Up,
    Down,
}

#[derive(PartialEq, Serialize, Deserialize)]
pub struct HoverState {
    pub direction: bool,
    pub upper_limit: f32,
    pub lower_limit: f32,
}
