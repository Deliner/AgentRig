use crate::shared::calling_agent::Caller;
use crate::shared::console;
use crate::shared::diagnostics::actionable_findings::{Diagnostic, Level, print_diagnostics};
use crate::shared::fs_with_context as fs;
use crate::structural_lint::{Format, Location, Mode, analyze};
use anyhow::Result;
use std::path::Path;

const CONFIG_SKILL: &str = "repair (set config_skill in the lint configuration)";

pub(super) struct Output<'a> {
    pub(super) format: Format,
    pub(super) warnings: bool,
    pub(super) mode: Mode,
    pub(super) rerun: Option<&'a str>,
    pub(super) caller: &'a Caller,
}
pub fn run(location: Location<'_>, caller: &Caller, format: Format, mode: Mode) -> Result<i32> {
    run_output(
        location,
        Output {
            format,
            warnings: false,
            mode,
            rerun: None,
            caller,
        },
    )
}
pub fn check(location: Location<'_>, caller: &Caller, rerun: &str) -> Result<i32> {
    run_output(
        location,
        Output {
            format: Format::Text,
            warnings: false,
            mode: Mode::Lint,
            rerun: Some(rerun),
            caller,
        },
    )
}
// DECISION: D049
// DECISION: D188
pub(super) fn run_output(location: Location<'_>, output: Output<'_>) -> Result<i32> {
    let Location { root, config: path } = location;
    let command = match output.mode {
        Mode::ConfigCheck => "lint-config-check",
        Mode::Lint => "lint",
    };
    let result = analyze(location, output.mode);
    let (mut diagnostics, config_error) = match result {
        Ok(mut items) => {
            for item in &mut items {
                item.skill = output.caller.skill(&item.skill);
            }
            (items, false)
        }
        Err(error) => (vec![configuration_error(path, output.caller, error)], true),
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
pub(super) fn configuration_error(
    path: &Path,
    caller: &Caller,
    error: anyhow::Error,
) -> Diagnostic {
    let skill = configuration_skill(path)
        .map_or_else(|| CONFIG_SKILL.to_owned(), |skill| caller.skill(&skill));
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
    let value: serde_json::Value = crate::shared::strict_yaml::decode(&source).ok()?;
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
