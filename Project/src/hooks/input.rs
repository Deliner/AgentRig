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

#[cfg(test)]
mod tests {
    use super::{object, text};
    use serde_json::json;

    #[test]
    fn missing_invalid_and_non_object_state_start_empty() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.json");
        assert_eq!(object(&path), json!({}));
        for contents in ["invalid", "null", "[]", "42", "\"text\""] {
            std::fs::write(&path, contents).unwrap();
            assert_eq!(object(&path), json!({}));
        }
        std::fs::write(&path, r#"{"session_id":"abc","offset":3}"#).unwrap();
        assert_eq!(object(&path), json!({"session_id": "abc", "offset": 3}));
    }

    #[test]
    fn event_text_preserves_strings_and_defaults_other_values() {
        let event = json!({"session_id": "  abc  ", "offset": 3, "empty": null});
        assert_eq!(text(&event, "session_id"), "  abc  ");
        for key in ["offset", "empty", "missing"] {
            assert_eq!(text(&event, key), "");
        }
        assert_eq!(text(&json!([]), "session_id"), "");
    }
}
