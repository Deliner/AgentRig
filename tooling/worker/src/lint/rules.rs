use anyhow::{Result, bail};
use serde_json::{Value, json};

// DECISION: D016
// Add a rule here and its measurement in lint::evaluate; config validation shares this registry.
pub fn target(kind: &str) -> Result<&'static str> {
    match kind {
        "nonblank-lines" => Ok("file"),
        "directory-entries" => Ok("directory"),
        _ => bail!("unsupported rule kind {kind}"),
    }
}
pub fn catalog() -> Value {
    json!([
        {"kind": "nonblank-lines", "target": "file", "languages": "any UTF-8 text",
         "extensions": "configurable", "metric": "nonempty lines, including comments"},
        {"kind": "directory-entries", "target": "directory", "languages": "any",
         "extensions": "not applicable", "metric": "immediate child names in the selected inventory"}
    ])
}
