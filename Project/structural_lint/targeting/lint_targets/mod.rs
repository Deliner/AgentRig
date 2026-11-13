mod threshold_overrides;

pub use threshold_overrides::effective;

use super::file_discovery::Inventory;
use crate::shared::paths::glob_patterns::globs;
use crate::structural_lint::rule_set::lint_config::{Rule, extension_matches};
use anyhow::Result;
use globset::GlobSet;
use std::path::Path;

// DECISION: D018
pub struct Selected<'a> {
    pub path: &'a Path,
    pub warning: Option<usize>,
    pub error: Option<usize>,
    pub overrides: Vec<usize>,
}
pub fn select<'a>(rule: &Rule, inventory: &'a Inventory) -> Result<Vec<Selected<'a>>> {
    let disabled = !rule.enabled;
    if disabled {
        return Ok(Vec::new());
    }
    let selector = Selector::new(rule)?;
    let file_rule = rule.target == "file";
    let targets: Vec<_> = if file_rule {
        inventory.files.iter().collect()
    } else {
        inventory.directories.keys().collect()
    };
    let mut output = Vec::new();
    for path in targets {
        let selected = selector.reason(rule, path).is_none();
        if selected {
            output.push(effective(rule, path, &selector.overrides)?);
        }
    }
    Ok(output)
}

pub struct Selector {
    include: GlobSet,
    exclude: GlobSet,
    overrides: Vec<GlobSet>,
}
// DECISION: D129
impl Selector {
    pub fn overrides(&self) -> &[GlobSet] {
        &self.overrides
    }
    pub fn new(rule: &Rule) -> Result<Self> {
        Ok(Self {
            include: globs(&rule.include)?,
            exclude: globs(&rule.exclude)?,
            overrides: rule
                .overrides
                .iter()
                .map(|entry| globs(&entry.include))
                .collect::<Result<_>>()?,
        })
    }
    pub fn reason(&self, rule: &Rule, path: &Path) -> Option<&'static str> {
        let disabled = !rule.enabled;
        let outside = !self.include.is_match(path);
        let excluded = self.exclude.is_match(path);
        let extension = rule.target == "file" && !extension_matches(path, &rule.extensions);
        if disabled {
            return Some("disabled");
        }
        if outside {
            return Some("include does not match");
        }
        if excluded {
            return Some("rule exclude matches");
        }
        if extension {
            return Some("extension does not match");
        }
        None
    }
}
