use anyhow::{Result, bail};
use serde_json::{Value, json};

// DECISION: D016
// DECISION: D017
pub fn syntax(kind: &str) -> bool {
    matches!(
        kind,
        "named-if-condition" | "function-lines" | "parameter-count"
    )
}
pub fn target(kind: &str) -> Result<&'static str> {
    match kind {
        "nonblank-lines" | "named-if-condition" | "function-lines" | "parameter-count" => {
            Ok("file")
        }
        "directory-entries" => Ok("directory"),
        _ => bail!("unsupported rule kind {kind}"),
    }
}
pub fn validate_extensions(kind: &str, extensions: &[String]) -> Result<()> {
    let unsupported = syntax(kind)
        && extensions
            .iter()
            .any(|ext| !matches!(ext.as_str(), ".rs" | ".py" | ".pyi"));
    if unsupported {
        bail!("{kind}: handlers support only .rs, .py and .pyi");
    }
    Ok(())
}
pub fn catalog() -> Value {
    let mut entries = vec![
        json!({"kind": "nonblank-lines", "target": "file", "languages": "any UTF-8 text",
         "extensions": "configurable", "metric": "nonempty lines, including comments"}),
        json!({"kind": "directory-entries", "target": "directory", "languages": "any",
         "extensions": "not applicable", "metric": "immediate child names in the selected inventory"}),
    ];
    for (kind, metric) in [
        (
            "named-if-condition",
            "one named value per boolean if condition",
        ),
        (
            "function-lines",
            "nonblank lines from signature through body, including comments",
        ),
        (
            "parameter-count",
            "declared input parameters, excluding method receivers",
        ),
    ] {
        entries.push(
            json!({"kind": kind, "target": "file", "languages": ["rust", "python"],
            "handlers": {"rust": [".rs"], "python": [".py", ".pyi"]}, "metric": metric}),
        );
    }
    Value::Array(entries)
}
