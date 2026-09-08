use serde_json::Value;
use std::{fs, path::Path};

// DECISION: D015
pub fn object(path: &Path) -> Value {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}))
}
pub fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}
pub use crate::paths::resolve;
pub use review_runner::artifacts::json::save as save_json;
pub use review_runner::vcs::git_context as git;

pub use crate::arguments::take_option;
