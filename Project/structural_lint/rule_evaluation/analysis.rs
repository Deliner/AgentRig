use super::{baseline, evaluation, location::Location};
use crate::shared::diagnostics::Diagnostic;
use crate::shared::paths::globs;
use crate::structural_lint::architecture;
use crate::structural_lint::rule_identities::{Kind, Structure};
use crate::structural_lint::targeting::{file_discovery, lint_targets};
use anyhow::Result;

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
    pub(in crate::structural_lint) fn config_check(self) -> bool {
        match self {
            Self::ConfigCheck => true,
            Self::Lint => false,
        }
    }
}

pub fn analyze(location: Location<'_>, mode: Mode) -> Result<Vec<Diagnostic>> {
    let Location { root, config: path } = location;
    location.load().and_then(|config| {
        let config_check = mode.config_check();
        if config_check {
            let inventory = file_discovery::maintained_inventory(root, &globs(&config.exclude)?)?;
            for rule in &config.rules {
                lint_targets::select(rule, &inventory)?;
                let architecture = rule.kind == Kind::Structure(Structure::DirectoryArchitecture);
                if architecture {
                    architecture::validate(rule, &inventory)?;
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
