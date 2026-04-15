#[derive(PartialEq)]
pub enum HoverDirection {
    Up,
    Down,
}

pub struct HoverState {
    pub direction: HoverDirection,
    pub upper_limit: f32,
    pub lower_limit: f32,
}
