pub mod architecture;
pub mod baseline;
mod evaluation;
pub mod languages;
mod lifting;
pub mod lint_files;
pub mod reporting;
pub mod rule_identities;
// DECISION: D007
pub mod rule_set;
pub mod targeting;
#[cfg(test)]
mod tests;

pub use crate::shared::diagnostics::Diagnostic;
use crate::shared::paths::globs;
use crate::structural_lint::rule_identities::structure::Structure;
use anyhow::Result;
use rule_set::lint_config;
use std::path::Path;
use targeting::{file_discovery, lint_targets};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Format {
    Json,
    Text,
}
// DECISION: D062
#[derive(Clone, Copy)]
pub enum Mode {
    ConfigCheck,
    Lint,
}
impl Mode {
    pub fn of(command: &str) -> Self {
        match command {
            "lint-config-check" => Self::ConfigCheck,
            _ => Self::Lint,
        }
    }
    fn config_check(self) -> bool {
        match self {
            Self::ConfigCheck => true,
            Self::Lint => false,
        }
    }
}

// DECISION: D016
// DECISION: D017
// DECISION: D018
#[derive(Clone, Copy)]
pub struct Location<'a> {
    pub root: &'a Path,
    pub config: &'a Path,
}
impl Location<'_> {
    pub fn load(self) -> Result<lint_config::Config> {
        let mut config: lint_config::Config = crate::shared::strict_yaml::read(self.config)?;
        let directory = self.config.parent().unwrap_or(self.root);
        let external = config
            .skill_root
            .as_ref()
            .map(|skills| directory.join(skills).canonicalize())
            .transpose()?;
        config.validate(external.as_deref().unwrap_or(self.root))?;
        if let Some(resources) = external {
            config.resolve_guidance(&resources);
        }
        Ok(config)
    }
}
fn analyze(location: Location<'_>, mode: Mode) -> Result<Vec<Diagnostic>> {
    let Location { root, config: path } = location;
    location.load().and_then(|config| {
        let config_check = mode.config_check();
        if config_check {
            let inventory = file_discovery::collect(root, &globs(&config.exclude)?)?;
            for rule in &config.rules {
                lint_targets::select(rule, &inventory)?;
                let architecture =
                    rule.kind == rule_identities::Kind::Structure(Structure::DirectoryArchitecture);
                if architecture {
                    architecture::contract_check::validate(rule, &inventory)?;
                }
            }
            Ok(Vec::new())
        } else {
            let mut diagnostics = evaluation::evaluate(root, &config)?;
            baseline::ratchet(path, &mut diagnostics)?;
            Ok(diagnostics)
        }
    })
}
