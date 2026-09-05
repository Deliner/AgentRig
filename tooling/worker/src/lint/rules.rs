use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::Path;

// DECISION: D018
const HANDLERS: &[(&str, &[&str])] = &[("rust", &[".rs"]), ("python", &[".py", ".pyi"])];
pub fn supports_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| supported_extension(&format!(".{ext}")))
}
fn supported_extension(ext: &str) -> bool {
    HANDLERS
        .iter()
        .any(|(_, extensions)| extensions.contains(&ext))
}
pub fn validate_includes(kind: &str, patterns: &[String]) -> Result<()> {
    let structural = !syntax(kind);
    if structural {
        return Ok(());
    }
    for pattern in patterns {
        if let Some(ext) = Path::new(pattern).extension().and_then(|ext| ext.to_str()) {
            let literal = !ext.contains(['*', '?', '[', ']', '{', '}', '\\']);
            if literal {
                validate_extensions(kind, &[format!(".{ext}")])?;
            }
        }
    }
    Ok(())
}

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
    let unsupported = syntax(kind) && extensions.iter().any(|ext| !supported_extension(ext));
    if unsupported {
        bail!(
            "{kind}: requested extensions {extensions:?} are incompatible; handlers support Rust (.rs) and Python (.py, .pyi)"
        );
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
            "handlers": HANDLERS.iter().map(|(name, extensions)| (*name, *extensions)).collect::<std::collections::BTreeMap<_, _>>(), "extensions": HANDLERS.iter().flat_map(|(_, extensions)| extensions.iter()).collect::<Vec<_>>(), "metric": metric}),
        );
    }
    Value::Array(entries)
}
