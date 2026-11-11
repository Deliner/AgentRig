pub mod architecture;
pub mod baseline;
pub mod catalog;
pub mod cli;
// DECISION: D007
pub mod config;
mod evaluation;
pub mod explain;
pub mod inventory;
pub mod languages;
pub mod rules;
mod selection;
#[cfg(test)]
mod tests;

use crate::shared::console;
pub use crate::shared::diagnostics::lint::Diagnostic;
use crate::shared::diagnostics::lint::{Level, print_diagnostics};
use crate::shared::fs;
use crate::shared::paths::patterns::globs;
use crate::structural_lint::rules::structure::Structure;
use anyhow::Result;
use std::path::Path;

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
const CONFIG_SKILL: &str = "repair (set config_skill in the lint configuration)";
#[derive(Clone, Copy)]
pub struct Location<'a> {
    pub root: &'a Path,
    pub config: &'a Path,
}
impl Location<'_> {
    pub fn load(self) -> Result<config::Config> {
        let mut config: config::Config = crate::shared::yaml::read(self.config)?;
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
struct Output<'a> {
    format: Format,
    warnings: bool,
    mode: Mode,
    rerun: Option<&'a str>,
}
pub fn run(location: Location<'_>, format: Format, mode: Mode) -> Result<i32> {
    run_output(
        location,
        Output {
            format,
            warnings: false,
            mode,
            rerun: None,
        },
    )
}
pub fn check(location: Location<'_>, rerun: &str) -> Result<i32> {
    run_output(
        location,
        Output {
            format: Format::Text,
            warnings: false,
            mode: Mode::Lint,
            rerun: Some(rerun),
        },
    )
}
// DECISION: D049
fn run_output(location: Location<'_>, output: Output<'_>) -> Result<i32> {
    let Location { root, config: path } = location;
    let command = match output.mode {
        Mode::ConfigCheck => "lint-config-check",
        Mode::Lint => "lint",
    };
    let result = analyze(location, output.mode);
    let (mut diagnostics, config_error) = match result {
        Ok(items) => (items, false),
        Err(error) => (vec![configuration_error(path, error)], true),
    };
    diagnostics.retain(|item| output.warnings || item.level.blocking());
    let args = rerun_args(command, path, &output);
    for item in &mut diagnostics {
        item.rerun = output
            .rerun
            .map(str::to_owned)
            .unwrap_or_else(|| crate::shared::diagnostics::rerun(root, &args));
    }
    let failed = diagnostics.iter().any(|item| item.level.blocking());
    match output.format {
        Format::Json => console::stdout(serde_json::to_string(&diagnostics)?),
        Format::Text => {
            let configuration_valid = output.mode.config_check() && !config_error;
            if configuration_valid {
                console::stdout(format!("Lint configuration is valid: {}", path.display()));
            }
            print_diagnostics(&diagnostics);
        }
    }
    Ok(if config_error { 2 } else { i32::from(failed) })
}
fn rerun_args(command: &str, path: &Path, output: &Output<'_>) -> Vec<String> {
    let mut args = vec![
        command.into(),
        "--config".into(),
        path.to_string_lossy().into_owned(),
    ];
    let json = match output.format {
        Format::Json => Some("--json"),
        Format::Text => None,
    };
    let warnings = output.warnings.then_some("--warnings");
    args.extend([json, warnings].into_iter().flatten().map(String::from));
    args
}
fn analyze(location: Location<'_>, mode: Mode) -> Result<Vec<Diagnostic>> {
    let Location { root, config: path } = location;
    location.load().and_then(|config| {
        let config_check = mode.config_check();
        if config_check {
            let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
            for rule in &config.rules {
                selection::select(rule, &inventory)?;
                let architecture =
                    rule.kind == rules::Kind::Structure(Structure::DirectoryArchitecture);
                if architecture {
                    architecture::runner::validate(rule, &inventory)?;
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
fn configuration_error(path: &Path, error: anyhow::Error) -> Diagnostic {
    let skill = configuration_skill(path).unwrap_or_else(|| CONFIG_SKILL.to_owned());
    Diagnostic {
        rule: "configuration".into(),
        path: path.display().to_string(),
        level: Level::Error,
        actual: None,
        limit: None,
        skill,
        message: format!("{error:#}"),
        line: None,
        symbol: None,
        rerun: String::new(),
    }
}
fn configuration_skill(path: &Path) -> Option<String> {
    let source = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = crate::shared::yaml::decode(&source).ok()?;
    let skill = value.get("config_skill")?.as_str()?;
    match value.get("skill_root").and_then(serde_json::Value::as_str) {
        Some(root) => {
            let candidate = path.parent()?.join(root).join(skill);
            Some(
                candidate
                    .canonicalize()
                    .unwrap_or(candidate)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
        None => Some(skill.into()),
    }
}
