pub mod config;
mod inventory;
pub mod rules;

use anyhow::{Result, bail};
use config::{extension_matches, globs};
use serde::Serialize;
use std::{fs, path::Path};

// DECISION: D016
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
}
fn evaluate(root: &Path, config: &config::Config) -> Result<Vec<Diagnostic>> {
    let inventory = inventory::collect(root, &globs(&config.exclude)?)?;
    let mut output = Vec::new();
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
                    warning = entry.warning.unwrap_or(warning);
                    error = entry.error.unwrap_or(error);
                }
            }
            if warning >= error {
                bail!(
                    "{} at {}: effective warning must be below error",
                    rule.id,
                    path.display()
                );
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
            let (level, limit, skill) = if actual > error {
                ("error", error, &rule.error_skill)
            } else if actual > warning {
                ("warning", warning, &rule.warning_skill)
            } else {
                continue;
            };
            output.push(Diagnostic {
                rule: rule.id.clone(),
                path: path.to_string_lossy().into_owned(),
                level: level.into(),
                actual: Some(actual),
                limit: Some(limit),
                skill: skill.clone(),
                message: format!("{} measures {actual}, exceeds {limit}", rule.kind),
            });
        }
    }
    Ok(output)
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
                item.path,
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
