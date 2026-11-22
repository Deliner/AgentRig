use super::schema::{Config, Override, Rule};
use crate::shared::{diagnostics::validate_skill, paths::globs};
use crate::structural_lint::rule_identities::{ControlFlow, Ownership, Pair, Structure};
use crate::structural_lint::rule_set::capabilities;
use anyhow::{Result, bail};
use std::{collections::HashSet, path::Path};

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
struct Checked<'a> {
    resources: &'a Path,
    skills: HashSet<&'a str>,
    globs: HashSet<&'a [String]>,
}
impl<'a> Checked<'a> {
    fn skill(&mut self, value: &'a str) -> Result<()> {
        let fresh = !self.skills.contains(value);
        if fresh {
            validate_skill(self.resources, value)?;
            self.skills.insert(value);
        }
        Ok(())
    }
    fn globs(&mut self, patterns: &'a [String]) -> Result<()> {
        let fresh = !self.globs.contains(patterns);
        if fresh {
            globs(patterns)?;
            self.globs.insert(patterns);
        }
        Ok(())
    }
}
impl Config {
    pub fn validate(&self, resources: &Path) -> Result<()> {
        let unsupported_version = self.version != 1;
        if unsupported_version {
            bail!("unsupported config version {}", self.version);
        }
        let mut checked = Checked {
            resources,
            skills: HashSet::new(),
            globs: HashSet::new(),
        };
        checked.skill(&self.config_skill)?;
        checked.globs(&self.exclude)?;
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
            rule.validate(&mut checked)?;
        }
        single_pairs(&self.rules)?;
        measured_dependencies(&self.rules)
    }
}
// DECISION: D225
// DECISION: D227
// DECISION: D228
fn measured_dependencies(rules: &[Rule]) -> Result<()> {
    let enabled = |kind: Structure| {
        let kind = capabilities::Kind::Structure(kind);
        rules.iter().any(|rule| rule.enabled && rule.kind == kind)
    };
    let measured = enabled(Structure::DirectoryArchitecture);
    for kind in [
        Structure::FileCycle,
        Structure::EntryUse,
        Structure::EntryBypass,
    ] {
        let unmeasured = enabled(kind) && !measured;
        if unmeasured {
            let kind = capabilities::Kind::Structure(kind);
            bail!("{kind} reads the dependencies of a directory-architecture rule; enable one");
        }
    }
    Ok(())
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
    fn validate<'a>(&'a self, checked: &mut Checked<'a>) -> Result<()> {
        let incompatible_target = self.kind.descriptor().target != self.target;
        if incompatible_target {
            bail!("{}: unsupported target {}", self.id, self.target);
        }
        let missing_selection = self.include.is_empty();
        if missing_selection {
            bail!("{}: include must select targets", self.id);
        }
        checked.globs(&self.include)?;
        checked.globs(&self.exclude)?;
        let architecture =
            self.kind == capabilities::Kind::Structure(Structure::DirectoryArchitecture);
        let extension_target = if architecture { "file" } else { &self.target };
        validate_extensions(&self.extensions, extension_target)?;
        self.validate_architecture()?;
        self.validate_junctions()?;
        self.validate_pairs()?;
        self.validate_languages()?;
        capabilities::validate_extensions(self.kind, &self.extensions)?;
        capabilities::validate_includes(self.kind, &self.include)?;
        self.validate_severity()?;
        checked.skill(&self.warning_skill)?;
        checked.skill(&self.error_skill)?;
        for entry in &self.overrides {
            self.validate_override(entry, checked)?;
        }
        Ok(())
    }
    fn validate_severity(&self) -> Result<()> {
        match self.kind.descriptor().parameters {
            capabilities::Parameters::Level { .. } => {
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
            capabilities::Parameters::Thresholds { .. } => {
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
        let architecture =
            self.kind == capabilities::Kind::Structure(Structure::DirectoryArchitecture);
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
        let boolean = self.kind == capabilities::Kind::ControlFlow(ControlFlow::BooleanExpression);
        let single = !self.junctions.mixed();
        let misplaced = single && !boolean;
        if misplaced {
            bail!("{}: junctions apply only to boolean-expression", self.id);
        }
        Ok(())
    }
    // DECISION: D156
    fn validate_pairs(&self) -> Result<()> {
        let paired = self.kind == capabilities::Kind::Ownership(Ownership::PairedCalls);
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
    // DECISION: D198
    fn validate_languages(&self) -> Result<()> {
        let program = self.kind == capabilities::Kind::Structure(Structure::ProgramLanguage);
        match (&self.allowed_languages, program) {
            (Some(names), true) => capabilities::validate_program_languages(&self.id, names)?,
            (None, true) => bail!("{}: program-language needs allowed_languages", self.id),
            (Some(_), false) => bail!("{}: allowed_languages require program-language", self.id),
            (None, false) => {}
        }
        Ok(())
    }
    fn validate_override<'a>(&self, entry: &'a Override, checked: &mut Checked<'a>) -> Result<()> {
        let thresholdless = entry.warning.is_none() && entry.error.is_none();
        let incomplete = entry.include.is_empty() || thresholdless;
        if incomplete {
            bail!("{}: override needs selectors and a threshold", self.id);
        }
        checked.globs(&entry.include)?;
        validate_extensions(&entry.extensions, &self.target)?;
        capabilities::validate_extensions(self.kind, &entry.extensions)?;
        capabilities::validate_includes(self.kind, &entry.include)?;
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
