use super::{evaluation, findings::clippy_finding, location::Location};
use crate::shared::diagnostics::Diagnostic;
use crate::shared::paths::globs;
use crate::structural_lint::architecture;
use crate::structural_lint::clippy::Clippy;
use crate::structural_lint::rule_identities::{Kind, Structure};
use crate::structural_lint::targeting::{file_discovery, lint_targets};
use anyhow::Result;

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Mode<'a> {
    ConfigCheck,
    Lint { clippy: Option<Clippy<'a>> },
}
impl Mode<'_> {
    pub fn of(command: &str) -> Self {
        match command {
            "lint-config-check" => Self::ConfigCheck,
            _ => Self::Lint { clippy: None },
        }
    }
    pub(in crate::structural_lint) fn config_check(self) -> bool {
        match self {
            Self::ConfigCheck => true,
            Self::Lint { .. } => false,
        }
    }
}

pub fn analyze(location: Location<'_>, mode: Mode<'_>) -> Result<Vec<Diagnostic>> {
    let root = location.root;
    location.load().and_then(|config| match mode {
        Mode::ConfigCheck => {
            let inventory = file_discovery::maintained_inventory(root, &globs(&config.exclude)?)?;
            for rule in &config.rules {
                lint_targets::select(rule, &inventory)?;
                let architecture = rule.kind == Kind::Structure(Structure::DirectoryArchitecture);
                if architecture {
                    architecture::validate(rule, &inventory)?;
                }
            }
            Ok(Vec::new())
        }
        Mode::Lint { clippy } => {
            let mut found = evaluation::evaluate(root, &config)?;
            let messages = clippy.map(|clippy| clippy.messages(root));
            let skill = &config.config_skill;
            let reported = messages.into_iter().flatten();
            found.extend(reported.map(|message| clippy_finding(message, skill)));
            Ok(found)
        }
    })
}

// DECISION: D235
pub fn error_count(location: Location<'_>) -> Result<usize> {
    let diagnostics = analyze(location, Mode::Lint { clippy: None })?;
    Ok(diagnostics
        .iter()
        .filter(|item| item.level.blocking())
        .count())
}
