use super::{
    config::{self, Rule, extension_matches, globs},
    inventory::Inventory,
    rules,
};
use anyhow::{Context, Result, bail};
use globset::GlobSet;
use std::path::Path;

// DECISION: D018
pub struct Selected<'a> {
    pub path: &'a Path,
    pub warning: Option<u64>,
    pub error: Option<u64>,
}
pub fn select<'a>(rule: &Rule, inventory: &'a Inventory) -> Result<Vec<Selected<'a>>> {
    let disabled = !rule.enabled;
    if disabled {
        return Ok(Vec::new());
    }
    let include = globs(&rule.include)?;
    let exclude = globs(&rule.exclude)?;
    let overrides = rule
        .overrides
        .iter()
        .map(|entry| globs(&entry.include))
        .collect::<Result<Vec<_>>>()?;
    let file_rule = rule.target == "file";
    let targets: Vec<_> = if file_rule {
        inventory.files.iter().collect()
    } else {
        inventory.directories.keys().collect()
    };
    let mut output = Vec::new();
    for path in targets {
        let selected = include.is_match(path)
            && !exclude.is_match(path)
            && extension_matches(path, &rule.extensions);
        if selected {
            output.push(effective(rule, path, &overrides)?);
        }
    }
    Ok(output)
}
fn effective<'a>(rule: &Rule, path: &'a Path, overrides: &[GlobSet]) -> Result<Selected<'a>> {
    let incompatible = rules::syntax(&rule.kind) && !rules::supports_path(path);
    if incompatible {
        bail!(
            "{} ({}): selected {} has no supported handler; supports Rust (.rs) and Python (.py, .pyi). Narrow extensions/include or exclude this path",
            rule.id,
            rule.kind,
            path.display()
        );
    }
    let (mut warning, mut error) = (rule.warning, rule.error);
    for (entry, selector) in rule.overrides.iter().zip(overrides) {
        let matches = selector.is_match(path) && extension_matches(path, &entry.extensions);
        if matches {
            warning = entry.warning.or(warning);
            error = entry.error.or(error);
        }
    }
    let numeric = rule.level.is_none();
    if numeric {
        config::thresholds(warning, error).with_context(|| {
            format!(
                "{} at {}: invalid effective thresholds",
                rule.id,
                path.display()
            )
        })?;
    }
    Ok(Selected {
        path,
        warning,
        error,
    })
}
