use std::sync::{Arc, Mutex};

use super::firestore::RemoteScoreEntry;

pub type FetchResult = Result<Vec<RemoteScoreEntry>, String>;
pub type SubmitResult = Result<(), String>;

pub struct AsyncHighScoreTask {
    pub fetch_result: Arc<Mutex<Option<FetchResult>>>,
    pub submit_result: Arc<Mutex<Option<SubmitResult>>>,
}

impl AsyncHighScoreTask {
    pub fn new() -> Self {
        Self {
            fetch_result: Arc::new(Mutex::new(None)),
            submit_result: Arc::new(Mutex::new(None)),
        }
    }
}
