use serde::{Deserialize, Serialize};

pub const MAX_ENTRIES: usize = 10;
pub const INITIALS_LEN: usize = 3;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct HighScoreEntry {
    pub initials: String,
    pub score: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct HighScores {
    pub entries: Vec<HighScoreEntry>,
}

impl HighScores {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn load() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                Self::new()
            } else {
                match std::fs::read_to_string(file_path()) {
                    Ok(text) => match ron::from_str::<HighScores>(&text) {
                        Ok(mut hs) => {
                            hs.normalize();
                            hs
                        }
                        Err(error) => {
                            log::warn!("HighScores: failed to parse RON ({error}); starting empty");
                            Self::new()
                        }
                    },
                    Err(_) => Self::new(),
                }
            }
        }
    }

    pub fn save(&self) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // WASM persistence deferred to remote DB work.
            } else {
                let path = file_path();
                if let Some(parent) = path.parent() {
                    if let Err(error) = std::fs::create_dir_all(parent) {
                        log::error!("HighScores: failed to create dir {parent:?}: {error}");
                        return;
                    }
                }
                match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
                    Ok(text) => {
                        if let Err(error) = std::fs::write(&path, text) {
                            log::error!("HighScores: failed to write {path:?}: {error}");
                        }
                    }
                    Err(error) => log::error!("HighScores: failed to serialise: {error}"),
                }
            }
        }
    }

    pub fn qualifies(&self, score: i32) -> bool {
        if score <= 0 {
            return false;
        }
        if self.entries.len() < MAX_ENTRIES {
            return true;
        }
        score > self.entries.last().map(|e| e.score).unwrap_or(i32::MIN)
    }

    /// Insert and re-sort. Returns the index of the newly placed entry, or
    /// `None` if it didn't make the cut.
    pub fn insert(&mut self, initials: &str, score: i32) -> Option<usize> {
        if !self.qualifies(score) {
            return None;
        }
        let initials = sanitize_initials(initials);
        self.entries.push(HighScoreEntry { initials: initials.clone(), score });
        self.normalize();
        self.entries
            .iter()
            .position(|e| e.score == score && e.initials == initials)
    }

    fn normalize(&mut self) {
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(MAX_ENTRIES);
    }
}

pub fn sanitize_initials(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .take(INITIALS_LEN)
        .collect();
    if cleaned.is_empty() {
        "AAA".to_string()
    } else {
        // Pad to 3 chars so the leaderboard renders consistently.
        format!("{:_<width$}", cleaned, width = INITIALS_LEN)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn file_path() -> std::path::PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(std::env::temp_dir);
    path.push("citizen-engine");
    path.push("high_scores.ron");
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifies_when_empty() {
        let hs = HighScores::new();
        assert!(hs.qualifies(1));
        assert!(!hs.qualifies(0));
    }

    #[test]
    fn insert_sorts_desc_and_truncates_to_ten() {
        let mut hs = HighScores::new();
        for i in 0..15 {
            hs.insert("ABC", i * 10);
        }
        assert_eq!(hs.entries.len(), MAX_ENTRIES);
        assert_eq!(hs.entries[0].score, 140);
        assert_eq!(hs.entries[9].score, 50);
    }

    #[test]
    fn qualifies_only_if_better_than_tenth_when_full() {
        let mut hs = HighScores::new();
        for i in 0..10 {
            hs.insert("AAA", 100 + i);
        }
        // tenth place is 100
        assert!(!hs.qualifies(100));
        assert!(hs.qualifies(101));
    }

    #[test]
    fn sanitize_pads_and_uppercases() {
        assert_eq!(sanitize_initials("ab"), "AB_");
        assert_eq!(sanitize_initials("abcdef"), "ABC");
        assert_eq!(sanitize_initials("a!b@c#"), "ABC");
        assert_eq!(sanitize_initials(""), "AAA");
    }
}
