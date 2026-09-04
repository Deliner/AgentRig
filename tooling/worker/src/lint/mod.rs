pub mod config;
mod inventory;
mod languages;
pub mod rules;

use anyhow::Result;
use config::{extension_matches, globs};
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path};

// DECISION: D016
// DECISION: D017
const CONFIG_SKILL: &str = ".agents/skills/configure-linter/SKILL.md";
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
}
fn evaluate(root: &Path, config: &config::Config) -> Result<Vec<Diagnostic>> {
    let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
    let mut output = Vec::new();
    let mut analyses = HashMap::new();
    for rule in &config.rules {
        let include = globs(&rule.include)?;
        let exclude = globs(&rule.exclude)?;
        let overrides = rule
            .overrides
            .iter()
            .map(|entry| globs(&entry.include))
            .collect::<Result<Vec<_>>>()?;
        let targets: Vec<_> = if rule.target == "file" {
            inventory.files.iter().collect()
        } else {
            inventory.directories.keys().collect()
        };
        for path in targets {
            if !include.is_match(path)
                || exclude.is_match(path)
                || !extension_matches(path, &rule.extensions)
            {
                continue;
            }
            let (mut warning, mut error) = (rule.warning, rule.error);
            for (entry, selector) in rule.overrides.iter().zip(&overrides) {
                if selector.is_match(path) && extension_matches(path, &entry.extensions) {
                    warning = entry.warning.or(warning);
                    error = entry.error.or(error);
                }
            }
            if rule.level.is_none() {
                config::thresholds(warning, error)?;
            }
            let syntax_rule = rules::syntax(&rule.kind);
            if syntax_rule {
                let supported = languages::supports(path);
                if !supported {
                    continue;
                }
                if !analyses.contains_key(path) {
                    let source = fs::read_to_string(root.join(path))?;
                    analyses.insert(path.clone(), languages::analyze(path, &source)?);
                }
                let analysis = &analyses[path];
                if let Some(line) = analysis.parse_error {
                    let reported = output.iter().any(|item: &Diagnostic| {
                        item.rule == "syntax" && item.path == path.to_string_lossy()
                    });
                    if !reported {
                        output.push(Diagnostic { rule: "syntax".into(), path: path.to_string_lossy().into_owned(),
                            level: "error".into(), actual: None, limit: None, skill: rule.error_skill.clone(),
                            message: "cannot analyze malformed syntax; repair source before evaluating rules".into(),
                            line: Some(line), symbol: None });
                    }
                    continue;
                }
                for measured in analysis
                    .measurements
                    .iter()
                    .filter(|item| item.kind == rule.kind)
                {
                    let mut diagnostic = match finding(rule, path, measured.actual, warning, error)?
                    {
                        Some(item) => item,
                        None => continue,
                    };
                    diagnostic.line = Some(measured.line);
                    diagnostic.symbol = Some(measured.symbol.clone());
                    output.push(diagnostic);
                }
                continue;
            }
            let actual = match rule.kind.as_str() {
                "nonblank-lines" => match String::from_utf8(fs::read(root.join(path))?) {
                    Ok(source) => source
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .count() as u64,
                    Err(_) => continue,
                },
                "directory-entries" => inventory.directories[path].len() as u64,
                _ => unreachable!("validated by rule registry"),
            };
            if let Some(item) = finding(rule, path, actual, warning, error)? {
                output.push(item);
            }
        }
    }
    Ok(output)
}
fn finding(
    rule: &config::Rule,
    path: &Path,
    actual: u64,
    warning: Option<u64>,
    error: Option<u64>,
) -> Result<Option<Diagnostic>> {
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
    Ok(Some(Diagnostic {
        rule: rule.id.clone(),
        path: path.to_string_lossy().into_owned(),
        level: level.into(),
        actual: Some(actual),
        limit,
        skill: if level == "error" {
            rule.error_skill.clone()
        } else {
            rule.warning_skill.clone()
        },
        message,
        line: None,
        symbol: None,
    }))
}
pub fn run(root: &Path, path: &Path, json: bool) -> Result<i32> {
    let config = config::load(root, path);
    let skill = config
        .as_ref()
        .map(|config| config.config_skill.as_str())
        .unwrap_or(CONFIG_SKILL)
        .to_owned();
    let result = config.and_then(|config| evaluate(root, &config));
    let (diagnostics, config_error) = match result {
        Ok(items) => (items, false),
        Err(error) => (
            vec![Diagnostic {
                rule: "configuration".into(),
                path: path.display().to_string(),
                level: "error".into(),
                actual: None,
                limit: None,
                skill,
                message: format!("{error:#}"),
                line: None,
                symbol: None,
            }],
            true,
        ),
    };
    let failed = diagnostics.iter().any(|item| item.level == "error");
    if json {
        println!("{}", serde_json::to_string(&diagnostics)?);
    } else {
        for item in &diagnostics {
            println!(
                "{} [{}] {}: {}. ACTION: {} {}",
                item.level.to_uppercase(),
                item.rule,
                format_location(item),
                item.message,
                if item.level == "warning" {
                    "Consider"
                } else {
                    "Apply"
                },
                item.skill
            );
        }
    }
    Ok(if config_error { 2 } else { i32::from(failed) })
}

fn format_location(item: &Diagnostic) -> String {
    match (&item.line, &item.symbol) {
        (Some(line), Some(symbol)) => format!("{}:{line} ({symbol})", item.path),
        (Some(line), None) => format!("{}:{line}", item.path),
        _ => item.path.clone(),
    }
}
