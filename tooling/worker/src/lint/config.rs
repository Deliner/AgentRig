use super::rules;
use anyhow::{Result, bail};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

// DECISION: D016
// DECISION: D017
// DECISION: D018
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub config_skill: String,
    pub skill_root: Option<PathBuf>,
    #[serde(default)]
    pub exclude: Vec<String>,
    pub rules: Vec<Rule>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    pub id: String,
    pub kind: rules::Kind,
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
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub include: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<u64>,
    pub error: Option<u64>,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
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
        let empty = pattern.trim().is_empty();
        if empty {
            bail!("empty glob");
        }
        let escapes_root =
            Path::new(pattern).is_absolute() || pattern.split('/').any(|part| part == "..");
        if escapes_root {
            bail!("globs must stay relative to the repository: {pattern}");
        }
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}
pub fn skill(root: &Path, value: &str) -> Result<()> {
    let absolute = Path::new(value).is_absolute();
    if absolute {
        bail!("skill paths must be relative to the repository: {value}");
    }
    let path = root.join(value).canonicalize()?;
    let invalid_location =
        !path.starts_with(root) || path.file_name().is_none_or(|name| name != "SKILL.md");
    if invalid_location {
        bail!("skill must reference a repository SKILL.md: {value}");
    }
    let source = fs::read_to_string(path)?;
    let invalid_frontmatter = !source.starts_with("---\n")
        || !source.contains("\nname: ")
        || !source.contains("\ndescription: ");
    if invalid_frontmatter {
        bail!("invalid skill frontmatter: {value}");
    }
    Ok(())
}
pub fn load(root: &Path, path: &Path) -> Result<Config> {
    let mut config: Config = review_runner::config::yaml::read(path)?;
    let external = config
        .skill_root
        .as_ref()
        .map(|directory| path.parent().unwrap_or(root).join(directory).canonicalize())
        .transpose()?;
    let resources = external.as_deref().unwrap_or(root);
    let unsupported_version = config.version != 1;
    if unsupported_version {
        bail!("unsupported config version {}", config.version);
    }
    skill(resources, &config.config_skill)?;
    globs(&config.exclude)?;
    let missing_rules = config.rules.is_empty();
    if missing_rules {
        bail!("at least one rule is required");
    }
    let mut ids = HashSet::new();
    for rule in &config.rules {
        let invalid_id = rule.id.trim().is_empty() || !ids.insert(&rule.id);
        if invalid_id {
            bail!("empty or duplicate rule ID");
        }
        rule.validate(resources)?;
    }
    if let Some(resources) = external {
        resolve_guidance(&mut config, &resources);
    }
    Ok(config)
}
fn resolve_guidance(config: &mut Config, resources: &Path) {
    config.config_skill = resources
        .join(&config.config_skill)
        .to_string_lossy()
        .into_owned();
    for rule in &mut config.rules {
        rule.warning_skill = resources
            .join(&rule.warning_skill)
            .to_string_lossy()
            .into_owned();
        rule.error_skill = resources
            .join(&rule.error_skill)
            .to_string_lossy()
            .into_owned();
    }
}
impl Rule {
    fn validate(&self, root: &Path) -> Result<()> {
        let incompatible_target = self.kind.descriptor().target != self.target;
        if incompatible_target {
            bail!("{}: unsupported target {}", self.id, self.target);
        }
        let missing_selection = self.include.is_empty();
        if missing_selection {
            bail!("{}: include must select targets", self.id);
        }
        globs(&self.include)?;
        globs(&self.exclude)?;
        validate_extensions(&self.extensions, &self.target)?;
        rules::validate_extensions(self.kind, &self.extensions)?;
        rules::validate_includes(self.kind, &self.include)?;
        self.validate_severity()?;
        skill(root, &self.warning_skill)?;
        skill(root, &self.error_skill)?;
        for entry in &self.overrides {
            self.validate_override(entry)?;
        }
        Ok(())
    }
    fn validate_severity(&self) -> Result<()> {
        let named_condition = matches!(self.kind.descriptor().parameters, rules::Parameters::Level);
        if named_condition {
            let invalid_level = self.level.is_none()
                || self.warning.is_some()
                || self.error.is_some()
                || !self.overrides.is_empty();
            if invalid_level {
                bail!(
                    "{}: named-if-condition needs level, no thresholds or overrides",
                    self.id
                );
            }
        } else {
            let has_level = self.level.is_some();
            if has_level {
                bail!("{}: numeric rules use thresholds, not level", self.id);
            }
            thresholds(self.warning, self.error)?;
        }
        Ok(())
    }
    fn validate_override(&self, entry: &Override) -> Result<()> {
        let incomplete =
            entry.include.is_empty() || (entry.warning.is_none() && entry.error.is_none());
        if incomplete {
            bail!("{}: override needs selectors and a threshold", self.id);
        }
        globs(&entry.include)?;
        validate_extensions(&entry.extensions, &self.target)?;
        rules::validate_extensions(self.kind, &entry.extensions)?;
        rules::validate_includes(self.kind, &entry.include)?;
        if let (Some(warning), Some(error)) = (entry.warning, entry.error) {
            let invalid_order = warning >= error;
            if invalid_order {
                bail!("{}: invalid override thresholds", self.id);
            }
        }
        Ok(())
    }
}
fn validate_extensions(extensions: &[String], target: &str) -> Result<()> {
    let directory_extensions = target == "directory" && !extensions.is_empty();
    if directory_extensions {
        bail!("directory rules do not accept extensions");
    }
    for ext in extensions {
        let invalid_suffix =
            !ext.starts_with('.') || ext.len() < 2 || ext[1..].contains(['/', '*', '?', '.']);
        if invalid_suffix {
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
