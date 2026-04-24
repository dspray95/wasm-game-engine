use serde::de::DeserializeOwned;

pub fn parse_ron_or_log<T: DeserializeOwned>(ron_str: &str, context: &str) -> Option<T> {
    match ron::from_str(ron_str) {
        Ok(value) => Some(value),
        Err(e) => {
            log::error!("Failed to parse {}: {}", context, e);
            None
        }
    }
}
