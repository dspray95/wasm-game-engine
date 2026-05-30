pub struct TutorialState {
    pub pressed_left: bool,
    pub pressed_right: bool,
    pub fired: bool,
    pub completed: bool,
}

impl TutorialState {
    pub fn new() -> Self {
        Self {
            pressed_left: false,
            pressed_right: false,
            fired: false,
            completed: false,
        }
    }

    pub fn move_done(&self) -> bool {
        self.pressed_left && self.pressed_right
    }
}
