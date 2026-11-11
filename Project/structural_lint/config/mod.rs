use super::catalog as rules;
pub use crate::shared::diagnostics::lint::Level;
use crate::shared::{diagnostics::skills, paths::patterns::globs};
use crate::structural_lint::rules::control_flow::{ControlFlow, Junctions};
use crate::structural_lint::rules::ownership::{Ownership, Pair};
use crate::structural_lint::rules::structure::Structure;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
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
    pub warning: Option<usize>,
    pub error: Option<usize>,
    pub level: Option<Level>,
    pub warning_skill: String,
    pub error_skill: String,
    #[serde(default)]
    pub overrides: Vec<Override>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<super::rules::architecture::Settings>,
    // DECISION: D156
    #[serde(default, skip_serializing_if = "Junctions::mixed")]
    pub junctions: Junctions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairs: Option<Vec<Pair>>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub include: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    pub warning: Option<usize>,
    pub error: Option<usize>,
}
fn enabled_by_default() -> bool {
    true
}
#[derive(Clone, Copy)]
pub struct Thresholds {
    pub warning: Option<usize>,
    pub error: Option<usize>,
}
impl Thresholds {
    pub fn validate(self) -> Result<()> {
        match (self.warning, self.error) {
            (None, None) => bail!("at least one warning/error threshold is required"),
            (Some(warning), Some(error)) if warning >= error => {
                bail!("warning must be below error")
            }
            _ => Ok(()),
        }
    }
}
impl Config {
    pub fn validate(&self, resources: &Path) -> Result<()> {
        let unsupported_version = self.version != 1;
        if unsupported_version {
            bail!("unsupported config version {}", self.version);
        }
        skills::reference(resources, &self.config_skill)?;
        globs(&self.exclude)?;
        let missing_rules = self.rules.is_empty();
        if missing_rules {
            bail!("at least one rule is required");
        }
        let mut ids = HashSet::new();
        for rule in &self.rules {
            let invalid_id = rule.id.trim().is_empty() || !ids.insert(&rule.id);
            if invalid_id {
                bail!("empty or duplicate rule ID");
            }
            rule.validate(resources)?;
        }
        single_pairs(&self.rules)
    }
    pub fn resolve_guidance(&mut self, resources: &Path) {
        self.config_skill = resources
            .join(&self.config_skill)
            .to_string_lossy()
            .into_owned();
        for rule in &mut self.rules {
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
}
// DECISION: D156
fn single_pairs(rules: &[Rule]) -> Result<()> {
    let repeated = rules.iter().filter(|rule| rule.pairs.is_some()).count() > 1;
    if repeated {
        bail!("at most one rule lists pairs");
    }
    Ok(())
}
impl Rule {
    pub fn skill(&self, level: Level) -> String {
        match level {
            Level::Warning => self.warning_skill.clone(),
            Level::Error => self.error_skill.clone(),
        }
    }
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
        let architecture = self.kind == rules::Kind::Structure(Structure::DirectoryArchitecture);
        let extension_target = if architecture { "file" } else { &self.target };
        validate_extensions(&self.extensions, extension_target)?;
        self.validate_architecture()?;
        self.validate_junctions()?;
        self.validate_pairs()?;
        rules::validate_extensions(self.kind, &self.extensions)?;
        rules::validate_includes(self.kind, &self.include)?;
        self.validate_severity()?;
        skills::reference(root, &self.warning_skill)?;
        skills::reference(root, &self.error_skill)?;
        for entry in &self.overrides {
            self.validate_override(entry)?;
        }
        Ok(())
    }
    fn validate_severity(&self) -> Result<()> {
        match self.kind.descriptor().parameters {
            rules::Parameters::Level { .. } => {
                let thresholds =
                    self.warning.is_some() || self.error.is_some() || !self.overrides.is_empty();
                let invalid_level = self.level.is_none() || thresholds;
                if invalid_level {
                    bail!(
                        "{}: {} needs level, no thresholds or overrides",
                        self.id,
                        self.kind
                    );
                }
            }
            rules::Parameters::Thresholds { .. } => {
                let has_level = self.level.is_some();
                if has_level {
                    bail!("{}: numeric rules use thresholds, not level", self.id);
                }
                let warning = self.warning;
                let error = self.error;
                Thresholds { warning, error }.validate()?;
            }
        }
        Ok(())
    }
    fn validate_architecture(&self) -> Result<()> {
        let architecture = self.kind == rules::Kind::Structure(Structure::DirectoryArchitecture);
        if architecture {
            let settings = self.architecture.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "{}: directory-architecture needs architecture settings",
                    self.id
                )
            })?;
            anyhow::ensure!(
                !self.extensions.is_empty(),
                "{}: architecture needs explicit source extensions",
                self.id
            );
            settings.validate()?;
        } else {
            anyhow::ensure!(
                self.architecture.is_none(),
                "{}: architecture settings require directory-architecture",
                self.id
            );
        }
        Ok(())
    }
    // DECISION: D156
    fn validate_junctions(&self) -> Result<()> {
        let boolean = self.kind == rules::Kind::ControlFlow(ControlFlow::BooleanExpression);
        let single = !self.junctions.mixed();
        let misplaced = single && !boolean;
        if misplaced {
            bail!("{}: junctions apply only to boolean-expression", self.id);
        }
        Ok(())
    }
    // DECISION: D156
    fn validate_pairs(&self) -> Result<()> {
        let paired = self.kind == rules::Kind::Ownership(Ownership::PairedCalls);
        match (&self.pairs, paired) {
            (Some(pairs), true) => {
                let valid = !pairs.is_empty() && pairs.iter().all(Pair::valid);
                anyhow::ensure!(
                    valid,
                    "{}: pairs need distinct nonempty open and close names",
                    self.id
                );
            }
            (None, true) => bail!("{}: paired-calls needs pairs", self.id),
            (Some(_), false) => bail!("{}: pairs require paired-calls", self.id),
            (None, false) => {}
        }
        Ok(())
    }
    fn validate_override(&self, entry: &Override) -> Result<()> {
        let thresholdless = entry.warning.is_none() && entry.error.is_none();
        let incomplete = entry.include.is_empty() || thresholdless;
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
        let invalid_suffix = ext
            .strip_prefix('.')
            .is_none_or(|suffix| suffix.is_empty() || suffix.contains(['/', '*', '?', '.']));
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
