use std::collections::HashSet;

use serde::Deserialize;

#[derive(Deserialize)]
struct ProfanityFile {
    words: Vec<String>,
}

pub struct ProfanityList {
    blocked: HashSet<String>,
}

impl ProfanityList {
    pub fn new() -> Self {
        Self { blocked: HashSet::new() }
    }

    pub fn from_ron(text: &str) -> Self {
        match ron::from_str::<ProfanityFile>(text) {
            Ok(file) => Self {
                blocked: file
                    .words
                    .into_iter()
                    .map(|word| word.to_ascii_uppercase())
                    .collect(),
            },
            Err(error) => {
                log::error!("ProfanityList: failed to parse RON: {error}");
                Self::new()
            }
        }
    }

    pub fn is_blocked(&self, candidate: &str) -> bool {
        self.blocked.contains(&candidate.to_ascii_uppercase())
    }
}
