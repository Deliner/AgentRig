pub mod architecture;
pub mod catalog;
pub mod cli;
// DECISION: D007
pub mod config;
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
use anyhow::Result;
use std::{collections::HashMap, path::Path};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Format {
    Json,
    Text,
}
impl Format {
    fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}
// DECISION: D062
#[derive(Clone, Copy)]
pub enum Mode {
    ConfigCheck,
    Lint,
}
impl Mode {
    fn config_check(self) -> bool {
        matches!(self, Self::ConfigCheck)
    }
}

// DECISION: D016
// DECISION: D017
// DECISION: D018
const CONFIG_SKILL: &str = "repair (set config_skill in the lint configuration)";
#[derive(Default)]
struct Evaluation {
    analyses: HashMap<std::path::PathBuf, languages::Analysis>,
    output: Vec<Diagnostic>,
}
fn evaluate(root: &Path, config: &config::Config) -> Result<Vec<Diagnostic>> {
    let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
    let mut evaluation = Evaluation::default();
    for rule in &config.rules {
        let architecture = rule.kind == rules::Kind::DirectoryArchitecture;
        if architecture {
            evaluation
                .output
                .extend(architecture::runner::run(root, rule, &inventory)?);
        } else {
            scalar_findings(root, rule, &inventory, &mut evaluation)?;
        }
    }
    Ok(evaluation.output)
}
fn scalar_findings(
    root: &Path,
    rule: &config::Rule,
    inventory: &inventory::Inventory,
    evaluation: &mut Evaluation,
) -> Result<()> {
    let Evaluation { analyses, output } = evaluation;
    for selected in selection::select(rule, inventory)? {
        let syntax_rule = catalog::syntax(rule.kind);
        if syntax_rule {
            let path = selected.path;
            let uncached = !analyses.contains_key(path);
            if uncached {
                let source = fs::read_to_string(root.join(path))?;
                analyses.insert(path.to_owned(), languages::analyze(path, &source)?);
            }
            syntax_findings(rule, &selected, &analyses[path], output)?;
        } else {
            for (line, actual) in structural_measurements(root, rule, &selected, inventory)? {
                if let Some(mut item) = finding(rule, &selected, actual)? {
                    item.line = line;
                    output.push(item);
                }
            }
        }
    }
    Ok(())
}
// DECISION: D072
const DUMP_NAMES: &[&str] = &["utils", "helpers", "misc", "common"];
// DECISION: D036
fn structural_measurements(
    root: &Path,
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    inventory: &inventory::Inventory,
) -> Result<Vec<(Option<usize>, u64)>> {
    let text = || -> Result<Option<String>> {
        Ok(String::from_utf8(fs::read(root.join(selected.path))?).ok())
    };
    Ok(match rule.kind {
        rules::Kind::NonblankLines => text()?
            .map(|source| {
                let count = source
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count();
                vec![(None, count as u64)]
            })
            .unwrap_or_default(),
        rules::Kind::LineWidth => text()?
            .map(|source| {
                source
                    .lines()
                    .enumerate()
                    .map(|(index, line)| (Some(index + 1), line.chars().count() as u64))
                    .collect()
            })
            .unwrap_or_default(),
        rules::Kind::ForbiddenFile => vec![(None, 1)],
        rules::Kind::DirectoryEntries => vec![(None, inventory.file_entries(selected.path) as u64)],
        rules::Kind::DumpName => {
            let name = selected.path.file_name().and_then(|name| name.to_str());
            let dump = name.is_some_and(|name| DUMP_NAMES.contains(&name));
            if dump { vec![(None, 1)] } else { Vec::new() }
        }
        _ => anyhow::bail!("rule kind {} has no structural metric", rule.kind),
    })
}
fn syntax_findings(
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    analysis: &languages::Analysis,
    output: &mut Vec<Diagnostic>,
) -> Result<()> {
    if let Some(line) = analysis.parse_error {
        let unreported = !output
            .iter()
            .any(|item| item.rule == "syntax" && item.path == selected.path.to_string_lossy());
        if unreported {
            output.push(syntax_error(rule, selected.path, line));
        }
        return Ok(());
    }
    for measured in analysis
        .measurements
        .iter()
        .filter(|item| item.kind == rule.kind)
    {
        if let Some(mut diagnostic) = finding(rule, selected, measured.actual)? {
            diagnostic.line = Some(measured.line);
            diagnostic.symbol = Some(measured.symbol.clone());
            output.push(diagnostic);
        }
    }
    Ok(())
}
fn syntax_error(rule: &config::Rule, path: &Path, line: usize) -> Diagnostic {
    Diagnostic {
        rule: "syntax".into(),
        path: path.to_string_lossy().into_owned(),
        level: Level::Error,
        actual: None,
        limit: None,
        skill: rule.error_skill.clone(),
        message: "cannot analyze malformed syntax; repair source before evaluating rules".into(),
        line: Some(line),
        symbol: None,
        rerun: String::new(),
    }
}
fn finding(
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    actual: u64,
) -> Result<Option<Diagnostic>> {
    let (warning, error) = (selected.warning, selected.error);
    let (level, limit) = if let Some(level) = rule.level {
        (level, None)
    } else {
        config::thresholds(warning, error)?;
        match (warning, error) {
            (_, Some(limit)) if actual > limit => (Level::Error, Some(limit)),
            (Some(limit), _) if actual > limit => (Level::Warning, Some(limit)),
            _ => return Ok(None),
        }
    };
    let message = rule_message(rule, limit, actual)?;
    let blocking = level == Level::Error;
    Ok(Some(Diagnostic {
        rule: rule.id.clone(),
        path: selected.path.to_string_lossy().into_owned(),
        level,
        actual: Some(actual),
        limit,
        skill: if blocking {
            rule.error_skill.clone()
        } else {
            rule.warning_skill.clone()
        },
        message,
        line: None,
        symbol: None,
        rerun: String::new(),
    }))
}
fn rule_message(rule: &config::Rule, limit: Option<u64>, actual: u64) -> Result<String> {
    Ok(match (limit, &rule.kind.descriptor().parameters) {
        (Some(limit), _) => format!("{} measures {actual}, exceeds {limit}", rule.kind),
        (None, rules::Parameters::Level { violation }) => (*violation).into(),
        (None, rules::Parameters::Thresholds { .. }) => {
            anyhow::bail!("threshold rule reported no limit")
        }
    })
}
struct Output<'a> {
    json: bool,
    warnings: bool,
    validate_only: bool,
    rerun: Option<&'a str>,
}
pub fn run(root: &Path, path: &Path, format: Format, mode: Mode) -> Result<i32> {
    run_output(
        root,
        path,
        Output {
            json: format.is_json(),
            warnings: false,
            validate_only: mode.config_check(),
            rerun: None,
        },
    )
}
pub fn check(root: &Path, path: &Path, rerun: &str) -> Result<i32> {
    run_output(
        root,
        path,
        Output {
            json: false,
            warnings: false,
            validate_only: false,
            rerun: Some(rerun),
        },
    )
}
// DECISION: D049
fn run_output(root: &Path, path: &Path, output: Output<'_>) -> Result<i32> {
    let (mode, command) = if output.validate_only {
        (Mode::ConfigCheck, "lint-config-check")
    } else {
        (Mode::Lint, "lint")
    };
    let result = analyze(root, path, mode);
    let (mut diagnostics, config_error) = match result {
        Ok(items) => (items, false),
        Err(error) => (vec![configuration_error(path, error)], true),
    };
    diagnostics.retain(|item| output.warnings || item.level != Level::Warning);
    let args = vec![
        command.into(),
        "--config".into(),
        path.to_string_lossy().into_owned(),
    ];
    for item in &mut diagnostics {
        item.rerun = output
            .rerun
            .map(str::to_owned)
            .unwrap_or_else(|| crate::shared::diagnostics::rerun(root, &args));
    }
    let failed = diagnostics.iter().any(|item| item.level == Level::Error);
    if output.json {
        console::stdout(serde_json::to_string(&diagnostics)?);
    } else {
        let configuration_valid = output.validate_only && !config_error;
        if configuration_valid {
            console::stdout(format!("Lint configuration is valid: {}", path.display()));
        }
        print_diagnostics(&diagnostics);
    }
    Ok(if config_error { 2 } else { i32::from(failed) })
}
fn analyze(root: &Path, path: &Path, mode: Mode) -> Result<Vec<Diagnostic>> {
    config::load(root, path).and_then(|config| {
        let config_check = mode.config_check();
        if config_check {
            let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
            for rule in &config.rules {
                selection::select(rule, &inventory)?;
                let architecture = rule.kind == rules::Kind::DirectoryArchitecture;
                if architecture {
                    architecture::runner::validate(rule, &inventory)?;
                }
            }
            Ok(Vec::new())
        } else {
            evaluate(root, &config)
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
