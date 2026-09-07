pub mod architecture;
pub mod cli;
// DECISION: D007
pub mod config;
pub mod explain;
pub mod inventory;
pub mod languages;
pub mod rules;
mod selection;

use anyhow::Result;
use config::globs;
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path};

// DECISION: D016
// DECISION: D017
// DECISION: D018
const CONFIG_SKILL: &str = "repair (set config_skill in the lint configuration)";
#[derive(Serialize)]
pub struct Diagnostic {
    rule: String,
    path: String,
    level: String,
    actual: Option<u64>,
    limit: Option<u64>,
    skill: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    rerun: String,
}
#[derive(Default)]
struct Evaluation {
    analyses: HashMap<std::path::PathBuf, languages::Analysis>,
    output: Vec<Diagnostic>,
}
fn evaluate(
    root: &Path,
    config: &config::Config,
    source: Option<&review_runner::vcs::Source<'_>>,
) -> Result<Vec<Diagnostic>> {
    let inventory = inventory::collect_source(root, &globs(&config.exclude)?, source)?;
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
        let syntax_rule = rules::syntax(rule.kind);
        if syntax_rule {
            let path = selected.path;
            let uncached = !analyses.contains_key(path);
            if uncached {
                let source = fs::read_to_string(root.join(path))?;
                analyses.insert(path.to_owned(), languages::analyze(path, &source)?);
            }
            syntax_findings(rule, &selected, &analyses[path], output)?;
        } else {
            let Some(actual) = structural_measurement(root, rule, &selected, inventory)? else {
                continue;
            };
            if let Some(item) = finding(rule, &selected, actual)? {
                output.push(item);
            }
        }
    }
    Ok(())
}
fn structural_measurement(
    root: &Path,
    rule: &config::Rule,
    selected: &selection::Selected<'_>,
    inventory: &inventory::Inventory,
) -> Result<Option<u64>> {
    Ok(match rule.kind {
        rules::Kind::NonblankLines => String::from_utf8(fs::read(root.join(selected.path))?)
            .ok()
            .map(|source| {
                source
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .count() as u64
            }),
        rules::Kind::DirectoryEntries => Some(inventory.directories[selected.path].len() as u64),
        _ => unreachable!("validated by rule registry"),
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
        level: "error".into(),
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
        (level.name(), None)
    } else {
        config::thresholds(warning, error)?;
        match (warning, error) {
            (_, Some(limit)) if actual > limit => ("error", Some(limit)),
            (Some(limit), _) if actual > limit => ("warning", Some(limit)),
            _ => return Ok(None),
        }
    };
    let message = match limit {
        Some(limit) => format!("{} measures {actual}, exceeds {limit}", rule.kind),
        None => "if condition must be one named value; give the branch reason a name".into(),
    };
    let blocking = level == "error";
    Ok(Some(Diagnostic {
        rule: rule.id.clone(),
        path: selected.path.to_string_lossy().into_owned(),
        level: level.into(),
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
struct Output<'a> {
    json: bool,
    validate_only: bool,
    rerun: Option<&'a str>,
    source: Option<&'a review_runner::vcs::Source<'a>>,
}
pub fn run(root: &Path, path: &Path, json: bool, validate_only: bool) -> Result<i32> {
    run_output(
        root,
        path,
        Output {
            json,
            validate_only,
            rerun: None,
            source: None,
        },
    )
}
pub fn check(
    root: &Path,
    path: &Path,
    rerun: &str,
    source: Option<&review_runner::vcs::Source<'_>>,
) -> Result<i32> {
    run_output(
        root,
        path,
        Output {
            json: false,
            validate_only: false,
            rerun: Some(rerun),
            source,
        },
    )
}
fn run_output(root: &Path, path: &Path, output: Output<'_>) -> Result<i32> {
    let result = analyze(root, path, output.validate_only, output.source);
    let (mut diagnostics, config_error) = match result {
        Ok(items) => (items, false),
        Err(error) => (vec![configuration_error(path, error)], true),
    };
    let command = if output.validate_only {
        "lint-config-check"
    } else {
        "lint"
    };
    let args = vec![
        command.into(),
        "--config".into(),
        path.to_string_lossy().into_owned(),
    ];
    for item in &mut diagnostics {
        item.rerun = output
            .rerun
            .map(str::to_owned)
            .unwrap_or_else(|| crate::diagnostics::rerun(root, &args));
    }
    let failed = diagnostics.iter().any(|item| item.level == "error");
    if output.json {
        println!("{}", serde_json::to_string(&diagnostics)?);
    } else {
        let configuration_valid = output.validate_only && !config_error;
        if configuration_valid {
            println!("Lint configuration is valid: {}", path.display());
        }
        for item in &diagnostics {
            print_diagnostic(item);
        }
    }
    Ok(if config_error { 2 } else { i32::from(failed) })
}
fn analyze(
    root: &Path,
    path: &Path,
    validate_only: bool,
    source: Option<&review_runner::vcs::Source<'_>>,
) -> Result<Vec<Diagnostic>> {
    config::load(root, path).and_then(|config| {
        if validate_only {
            let inventory = inventory::collect_source(root, &globs(&config.exclude)?, source)?;
            for rule in &config.rules {
                selection::select(rule, &inventory)?;
                let architecture = rule.kind == rules::Kind::DirectoryArchitecture;
                if architecture {
                    architecture::runner::validate(rule, &inventory)?;
                }
            }
            Ok(Vec::new())
        } else {
            evaluate(root, &config, source)
        }
    })
}
fn configuration_error(path: &Path, error: anyhow::Error) -> Diagnostic {
    // Resolve guidance independently of rule validation, including invalid rules.
    let skill = configuration_skill(path).unwrap_or_else(|| CONFIG_SKILL.to_owned());
    Diagnostic {
        rule: "configuration".into(),
        path: path.display().to_string(),
        level: "error".into(),
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
    let value: serde_json::Value = review_runner::config::yaml::decode(&source).ok()?;
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
fn print_diagnostic(item: &Diagnostic) {
    println!(
        "{}",
        crate::diagnostics::Guidance {
            level: &item.level,
            id: &item.rule,
            location: &format_location(item),
            message: &item.message,
            skill: &item.skill,
            rerun: &item.rerun,
        }
    );
}

fn format_location(item: &Diagnostic) -> String {
    match (&item.line, &item.symbol) {
        (Some(line), Some(symbol)) => format!("{}:{line} ({symbol})", item.path),
        (Some(line), None) => format!("{}:{line}", item.path),
        _ => item.path.clone(),
    }
}
