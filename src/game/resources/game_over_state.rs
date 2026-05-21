use crate::game::resources::high_scores::INITIALS_LEN;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameOverPhase {
    Playing,
    DeathRamping,
    EnteringInitials,
    Showing,
}

pub struct GameOverState {
    pub phase: GameOverPhase,
    pub initials_buffer: String,
    pub final_score: i32,
    pub submitted_index: Option<usize>,
    pub restart_requested: bool,
    pub entry_error: Option<&'static str>,
}

impl GameOverState {
    pub fn new() -> Self {
        Self {
            phase: GameOverPhase::Playing,
            initials_buffer: String::with_capacity(INITIALS_LEN),
            final_score: 0,
            submitted_index: None,
            restart_requested: false,
            entry_error: None,
        }
    }

    pub fn reset(&mut self) {
        self.phase = GameOverPhase::Playing;
        self.initials_buffer.clear();
        self.final_score = 0;
        self.submitted_index = None;
        self.restart_requested = false;
        self.entry_error = None;
    }
}
