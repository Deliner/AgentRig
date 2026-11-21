use crate::shared::fs_with_context as fs;
use crate::shared::paths::resolve;
use serde_json::Value;
use std::path::{Path, PathBuf};

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
pub fn paths(event: &Value) -> Vec<String> {
    let payload = &event["tool_input"];
    let mut paths = Vec::new();
    for key in ["file_path", "path", "notebook_path"] {
        if let Some(path) = payload[key].as_str() {
            paths.push(path.into());
        }
    }
    let patches = [
        payload.as_str(),
        payload["patch"].as_str(),
        payload["command"].as_str(),
        payload["input"].as_str(),
        payload["patch_text"].as_str(),
    ];
    for patch in patches.into_iter().flatten() {
        for line in patch.lines() {
            for prefix in [
                "*** Add File: ",
                "*** Update File: ",
                "*** Delete File: ",
                "*** Move to: ",
            ] {
                if let Some(path) = line.strip_prefix(prefix) {
                    paths.push(path.into());
                }
            }
        }
    }
    paths
}
pub fn target(root: &Path, event: &Value, path: &str) -> Option<PathBuf> {
    let cwd = root.join(text(event, "cwd"));
    let resolved = resolve(&cwd.join(path)).ok()?;
    resolved.strip_prefix(root).ok().map(Path::to_owned)
}
