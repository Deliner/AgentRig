use super::Selected;
use crate::structural_lint::rule_set::capabilities::{support, supports_path, syntax};
use crate::structural_lint::rule_set::lint_config::{self, Rule, extension_matches};
use anyhow::{Context, Result, bail};
use globset::GlobSet;
use std::path::Path;

pub fn effective<'a>(rule: &Rule, path: &'a Path, overrides: &[GlobSet]) -> Result<Selected<'a>> {
    let incompatible =
        rule.target == "file" && syntax(rule.kind) && !supports_path(rule.kind, path);
    if incompatible {
        bail!(
            "{} ({}): selected {} has no supported handler; supports {}. Narrow extensions/include or exclude this path",
            rule.id,
            rule.kind,
            path.display(),
            support(rule.kind)
        );
    }
    let (mut warning, mut error) = (rule.warning, rule.error);
    let mut matched = Vec::new();
    for (index, (entry, selector)) in rule.overrides.iter().zip(overrides).enumerate() {
        let matches = selector.is_match(path) && extension_matches(path, &entry.extensions);
        if matches {
            matched.push(index);
            warning = entry.warning.or(warning);
            error = entry.error.or(error);
        }
    }
    let numeric = rule.level.is_none();
    if numeric {
        let thresholds = lint_config::Thresholds { warning, error };
        thresholds.validate().with_context(|| {
            format!(
                "{id} at {path}: invalid effective thresholds",
                id = rule.id,
                path = path.display()
            )
        })?;
    }
    Ok(Selected {
        path,
        warning,
        error,
        overrides: matched,
    })
}
