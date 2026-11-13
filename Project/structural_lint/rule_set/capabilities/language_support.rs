use super::Kind;
use crate::structural_lint::languages;
use anyhow::{Result, bail};
use std::path::Path;

pub fn syntax(kind: Kind) -> bool {
    languages::HANDLERS
        .iter()
        .any(|handler| handler.rules.contains(&kind))
}
pub fn supports_path(kind: Kind, path: &Path) -> bool {
    languages::handler(path).is_some_and(|handler| handler.rules.contains(&kind))
}
pub fn extensions(kind: Kind) -> Vec<&'static str> {
    languages::HANDLERS
        .iter()
        .filter(|handler| handler.rules.contains(&kind))
        .flat_map(|handler| handler.extensions.iter().copied())
        .collect()
}
pub fn support(kind: Kind) -> String {
    languages::HANDLERS
        .iter()
        .filter(|handler| handler.rules.contains(&kind))
        .map(|handler| {
            format!(
                "{title} ({extensions})",
                title = handler.title,
                extensions = handler.extensions.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(" and ")
}
pub fn validate_includes(kind: Kind, patterns: &[String]) -> Result<()> {
    let structural = !syntax(kind) || kind.descriptor().target == "directory";
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
pub fn validate_extensions(kind: Kind, selected: &[String]) -> Result<()> {
    let supported = extensions(kind);
    let unsupported = syntax(kind)
        && selected
            .iter()
            .any(|ext| !supported.contains(&ext.as_str()));
    if unsupported {
        bail!(
            "{kind}: requested extensions {selected:?} are incompatible; handlers support {}",
            support(kind)
        );
    }
    Ok(())
}
