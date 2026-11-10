use crate::shared::arguments::take_option;
use crate::shared::console;
use crate::structural_lint::rules::structure::Structure;
use anyhow::{Result, ensure};
use std::{
    env,
    path::{Path, PathBuf},
};

// DECISION: D049
pub fn execute(root: &Path, config: &Path, args: &[String], mode: super::Mode) -> Result<i32> {
    ensure!(
        args.iter()
            .all(|arg| matches!(arg.as_str(), "--json" | "--warnings")),
        "unknown lint argument"
    );
    let json = args.iter().any(|arg| arg == "--json");
    let format = if json {
        super::Format::Json
    } else {
        super::Format::Text
    };
    super::run_output(
        root,
        config,
        super::Output {
            format,
            warnings: args.iter().any(|arg| arg == "--warnings"),
            mode,
            rerun: None,
        },
    )
}
pub fn failure(
    root: &Path,
    path: &Path,
    error: anyhow::Error,
    invocation: &[String],
) -> Result<i32> {
    let json = invocation.iter().any(|arg| arg == "--json");
    let text = !json;
    if text {
        return Err(error);
    }
    let mut finding = super::configuration_error(path, error);
    finding.rerun = crate::shared::diagnostics::rerun(root, invocation);
    console::stdout(serde_json::to_string(&[finding])?);
    Ok(2)
}
pub fn standalone(mut args: Vec<String>) -> Result<i32> {
    let command = args
        .first()
        .filter(|arg| !arg.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "lint".into());
    let explicit_command = args.first() == Some(&command);
    if explicit_command {
        args.remove(0);
    }
    let rules = matches!(command.as_str(), "lint-rules" | "lint-rule");
    if rules {
        return discovery(&command, &args);
    }
    ensure!(
        matches!(
            command.as_str(),
            "lint" | "lint-config-check" | "lint-explain" | "lint-edges"
        ),
        "expected lint, lint-config-check, lint-explain, lint-edges, lint-rule or lint-rules"
    );
    let root = match take_option(&mut args, "--root")? {
        Some(value) => PathBuf::from(value),
        None => env::current_dir()?,
    };
    let root = crate::shared::fs::canonicalize(root)?;
    let config = take_option(&mut args, "--config")?.unwrap_or_else(|| "lint.yaml".into());
    // DECISION: D143
    if let Some(inspect) = super::explain::inspection(&command) {
        return inspect(&root, &root.join(config), &args);
    }
    let config_check = command == "lint-config-check";
    let mode = if config_check {
        super::Mode::ConfigCheck
    } else {
        super::Mode::Lint
    };
    execute(&root, &root.join(config), &args, mode)
}

pub fn discovery(command: &str, args: &[String]) -> Result<i32> {
    let catalog = command == "lint-rules";
    if catalog {
        ensure!(args.is_empty(), "lint-rules takes no arguments");
        console::stdout(super::catalog::catalog());
        return Ok(0);
    }
    let kind = args
        .first()
        .ok_or_else(|| anyhow::anyhow!("lint-rule requires a rule kind"))?;
    let kind: super::rules::Kind = serde_json::from_value(serde_json::json!(kind))?;
    ensure!(
        args.iter()
            .skip(1)
            .all(|arg| matches!(arg.as_str(), "--json" | "--example")),
        "unknown lint-rule argument"
    );
    let example = args.iter().any(|arg| arg == "--example");
    let json = args.iter().any(|arg| arg == "--json");
    if example {
        let value = serde_json::json!({"version": 1, "config_skill": ".agents/skills/repair/SKILL.md",
            "rules": [super::catalog::example(kind, ".agents/skills", &["src/**".into()])]});
        console::stdout(crate::shared::yaml::encode(&value)?);
    } else if json {
        console::stdout(super::catalog::describe(kind));
    } else {
        print_rule(kind);
    }
    Ok(0)
}

fn print_rule(kind: super::rules::Kind) {
    let descriptor = kind.descriptor();
    console::stdout(format!(
        "{kind}: {metric}\nTarget: {target}\nRepair skill: {skill}\nParameters: {parameters}",
        metric = descriptor.metric,
        target = descriptor.target,
        skill = descriptor.skill,
        parameters = descriptor.parameters.describe()
    ));
    let syntax = super::catalog::syntax(kind);
    if syntax {
        console::stdout(format!("Languages: {}", super::catalog::support(kind)));
    }
    let architecture = kind == super::rules::Kind::Structure(Structure::DirectoryArchitecture);
    if architecture {
        console::stdout(format!(
            "Architecture settings and limits: {}",
            super::architecture::Settings::describe()
        ));
    }
    console::stdout(format!(
        "Use lint-rule {kind} --example for a complete configuration example."
    ));
}
