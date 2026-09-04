use super::rules;
use anyhow::{Result, bail};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

// DECISION: D016
// DECISION: D017
// DECISION: D018
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub config_skill: String,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub gate_skills: std::collections::BTreeMap<String, String>,
    pub rules: Vec<Rule>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    pub id: String,
    pub kind: String,
    pub target: String,
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<u64>,
    pub error: Option<u64>,
    pub level: Option<Level>,
    pub warning_skill: String,
    pub error_skill: String,
    #[serde(default)]
    pub overrides: Vec<Override>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub include: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<u64>,
    pub error: Option<u64>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warning,
    Error,
}
impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}
fn enabled_by_default() -> bool {
    true
}
pub fn thresholds(warning: Option<u64>, error: Option<u64>) -> Result<()> {
    match (warning, error) {
        (None, None) => bail!("at least one warning/error threshold is required"),
        (Some(warning), Some(error)) if warning >= error => bail!("warning must be below error"),
        _ => Ok(()),
    }
}
pub fn globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if pattern.trim().is_empty() {
            bail!("empty glob");
        }
        if Path::new(pattern).is_absolute() || pattern.split('/').any(|part| part == "..") {
            bail!("globs must stay relative to the repository: {pattern}");
        }
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}
pub fn skill(root: &Path, value: &str) -> Result<()> {
    if Path::new(value).is_absolute() {
        bail!("skill paths must be relative to the repository: {value}");
    }
    let path = root.join(value).canonicalize()?;
    if !path.starts_with(root) || path.file_name().is_none_or(|name| name != "SKILL.md") {
        bail!("skill must reference a repository SKILL.md: {value}");
    }
    let source = fs::read_to_string(path)?;
    if !source.starts_with("---\n")
        || !source.contains("\nname: ")
        || !source.contains("\ndescription: ")
    {
        bail!("invalid skill frontmatter: {value}");
    }
    Ok(())
}
pub fn load(root: &Path, path: &Path) -> Result<Config> {
    let config: Config = toml::from_str(&fs::read_to_string(path)?)?;
    if config.version != 1 {
        bail!("unsupported config version {}", config.version);
    }
    skill(root, &config.config_skill)?;
    globs(&config.exclude)?;
    for value in config.gate_skills.values() {
        skill(root, value)?;
    }
    if config.rules.is_empty() {
        bail!("at least one rule is required");
    }
    let mut ids = HashSet::new();
    for rule in &config.rules {
        if rule.id.trim().is_empty() || !ids.insert(&rule.id) {
            bail!("empty or duplicate rule ID");
        }
        if rules::target(&rule.kind)? != rule.target {
            bail!("{}: unsupported target {}", rule.id, rule.target);
        }
        if rule.include.is_empty() {
            bail!("{}: include must select targets", rule.id);
        }
        globs(&rule.include)?;
        globs(&rule.exclude)?;
        validate_extensions(&rule.extensions, &rule.target)?;
        rules::validate_extensions(&rule.kind, &rule.extensions)?;
        rules::validate_includes(&rule.kind, &rule.include)?;
        if rule.kind == "named-if-condition" {
            if rule.level.is_none()
                || rule.warning.is_some()
                || rule.error.is_some()
                || !rule.overrides.is_empty()
            {
                bail!(
                    "{}: named-if-condition needs level, no thresholds or overrides",
                    rule.id
                );
            }
        } else {
            if rule.level.is_some() {
                bail!("{}: numeric rules use thresholds, not level", rule.id);
            }
            thresholds(rule.warning, rule.error)?;
        }
        skill(root, &rule.warning_skill)?;
        skill(root, &rule.error_skill)?;
        for entry in &rule.overrides {
            if entry.include.is_empty() || (entry.warning.is_none() && entry.error.is_none()) {
                bail!("{}: override needs selectors and a threshold", rule.id);
            }
            globs(&entry.include)?;
            validate_extensions(&entry.extensions, &rule.target)?;
            rules::validate_extensions(&rule.kind, &entry.extensions)?;
            rules::validate_includes(&rule.kind, &entry.include)?;
            if let (Some(warning), Some(error)) = (entry.warning, entry.error)
                && warning >= error
            {
                bail!("{}: invalid override thresholds", rule.id);
            }
        }
    }
    Ok(config)
}
fn validate_extensions(extensions: &[String], target: &str) -> Result<()> {
    if target == "directory" && !extensions.is_empty() {
        bail!("directory rules do not accept extensions");
    }
    for ext in extensions {
        if !ext.starts_with('.') || ext.len() < 2 || ext[1..].contains(['/', '*', '?', '.']) {
            bail!("extensions must be literal suffixes such as .rs: {ext}");
        }
    }
    Ok(())
}
pub fn extension_matches(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path.extension().is_some_and(|ext| {
            extensions
                .iter()
                .any(|wanted| wanted == &format!(".{}", ext.to_string_lossy()))
        })
}
