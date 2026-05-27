use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct RemoteScoreEntry {
    pub initials: String,
    pub score: i32,
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::RemoteScoreEntry;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window, catch)]
        async fn fetchHighScores(max_entries: u32) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(js_namespace = window, catch)]
        async fn submitHighScore(initials: String, score: i32) -> Result<JsValue, JsValue>;
    }

    pub async fn fetch_high_scores(max_entries: u32) -> Result<Vec<RemoteScoreEntry>, String> {
        let result = fetchHighScores(max_entries)
            .await
            .map_err(|error| format!("fetchHighScores JS error: {:?}", error))?;
        let json = result
            .as_string()
            .ok_or_else(|| "fetchHighScores: expected string result".to_string())?;
        serde_json::from_str::<Vec<RemoteScoreEntry>>(&json)
            .map_err(|error| format!("fetchHighScores: JSON parse error: {error}"))
    }

    pub async fn submit_high_score(initials: &str, score: i32) -> Result<(), String> {
        submitHighScore(initials.to_string(), score)
            .await
            .map(|_| ())
            .map_err(|error| format!("submitHighScore JS error: {:?}", error))
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{fetch_high_scores, submit_high_score};

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_high_scores(_max_entries: u32) -> Result<Vec<RemoteScoreEntry>, String> {
    Err("Not supported on native".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_high_score(_initials: &str, _score: i32) -> Result<(), String> {
    Err("Not supported on native".to_string())
}
