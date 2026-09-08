use super::{
    catalog as rules,
    config::{self, Rule, extension_matches, globs},
    inventory::Inventory,
};
use anyhow::{Context, Result, bail};
use globset::GlobSet;
use std::path::Path;

// DECISION: D018
pub struct Selected<'a> {
    pub path: &'a Path,
    pub warning: Option<u64>,
    pub error: Option<u64>,
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
pub fn effective<'a>(rule: &Rule, path: &'a Path, overrides: &[GlobSet]) -> Result<Selected<'a>> {
    let incompatible =
        rule.target == "file" && rules::syntax(rule.kind) && !rules::supports_path(rule.kind, path);
    if incompatible {
        bail!(
            "{} ({}): selected {} has no supported handler; supports {}. Narrow extensions/include or exclude this path",
            rule.id,
            rule.kind,
            path.display(),
            rules::support(rule.kind)
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
        overrides: matched,
    })
}

pub struct Selector {
    include: GlobSet,
    exclude: GlobSet,
    pub overrides: Vec<GlobSet>,
}
impl Selector {
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
