use super::findings::{Output, configuration_error, run_output};
use crate::shared::calling_agent::{Agent, Caller};
use crate::shared::console;
use crate::shared::shared_options::take_option;
use crate::structural_lint::rule_identities;
use crate::structural_lint::rule_identities::structure::Structure;
use crate::structural_lint::rule_set::capabilities;
use crate::structural_lint::{Format, Location, Mode};
use anyhow::{Result, ensure};
use std::{env, path::PathBuf};

pub type Inspection = fn(Location<'_>, &[String]) -> Result<i32>;

// DECISION: D143
pub fn inspection(command: &str) -> Option<Inspection> {
    match command {
        "lint-explain" => Some(super::explain::run),
        "lint-edges" => Some(super::explain::edges),
        "lint-tree" => Some(super::tree::run),
        _ => None,
    }
}

// DECISION: D049
pub fn execute(
    location: Location<'_>,
    caller: &Caller,
    args: &[String],
    mode: Mode,
) -> Result<i32> {
    ensure!(
        args.iter()
            .all(|arg| matches!(arg.as_str(), "--json" | "--warnings")),
        "unknown lint argument"
    );
    let json = args.iter().any(|arg| arg == "--json");
    let format = if json { Format::Json } else { Format::Text };
    run_output(
        location,
        Output {
            format,
            warnings: args.iter().any(|arg| arg == "--warnings"),
            mode,
            rerun: None,
            caller,
        },
    )
}
pub fn failure(
    location: Location<'_>,
    caller: &Caller,
    error: anyhow::Error,
    invocation: &[String],
) -> Result<i32> {
    let json = invocation.iter().any(|arg| arg == "--json");
    let text = !json;
    if text {
        return Err(error);
    }
    let mut finding = configuration_error(location.config, caller, error);
    finding.rerun = crate::shared::diagnostics::rerun(location.root, invocation);
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
            "lint" | "lint-config-check" | "lint-explain" | "lint-edges" | "lint-tree"
        ),
        "expected lint, lint-config-check, lint-explain, lint-edges, lint-tree, lint-rule or lint-rules"
    );
    let root = match take_option(&mut args, "--root")? {
        Some(value) => PathBuf::from(value),
        None => env::current_dir()?,
    };
    let root = crate::shared::fs_with_context::canonicalize(root)?;
    let config = take_option(&mut args, "--config")?.unwrap_or_else(|| "lint.yaml".into());
    let config = root.join(config);
    let location = Location {
        root: &root,
        config: &config,
    };
    // DECISION: D143
    if let Some(inspect) = inspection(&command) {
        return inspect(location, &args);
    }
    // DECISION: D188
    let caller = crate::shared::diagnostics::caller(&root, Agent::calling());
    execute(location, &caller, &args, Mode::of(&command))
}

pub fn discovery(command: &str, args: &[String]) -> Result<i32> {
    let catalog = command == "lint-rules";
    if catalog {
        ensure!(args.is_empty(), "lint-rules takes no arguments");
        console::stdout(capabilities::catalog());
        return Ok(0);
    }
    let kind = args
        .first()
        .ok_or_else(|| anyhow::anyhow!("lint-rule requires a rule kind"))?;
    let kind: rule_identities::Kind = serde_json::from_value(serde_json::json!(kind))?;
    ensure!(
        args.iter()
            .skip(1)
            .all(|arg| matches!(arg.as_str(), "--json" | "--example")),
        "unknown lint-rule argument"
    );
    let example = args.iter().any(|arg| arg == "--example");
    let json = args.iter().any(|arg| arg == "--json");
    if example {
        let value = serde_json::json!({"version": 1, "config_skill": ".agentrig/skills/repair/SKILL.md",
            "rules": [capabilities::example(kind, ".agentrig/skills", &["src/**".into()])]});
        console::stdout(crate::shared::strict_yaml::encode(&value)?);
    } else if json {
        console::stdout(capabilities::describe(kind));
    } else {
        print_rule(kind);
    }
    Ok(0)
}

fn print_rule(kind: rule_identities::Kind) {
    let descriptor = kind.descriptor();
    console::stdout(format!(
        "{kind}: {metric}\nTarget: {target}\nRepair skill: {skill}\nParameters: {parameters}",
        metric = descriptor.metric,
        target = descriptor.target,
        skill = descriptor.skill,
        parameters = descriptor.parameters.describe()
    ));
    let syntax = capabilities::syntax(kind);
    if syntax {
        console::stdout(format!("Languages: {}", capabilities::support(kind)));
    }
    let architecture = kind == rule_identities::Kind::Structure(Structure::DirectoryArchitecture);
    if architecture {
        console::stdout(format!(
            "Architecture settings and limits: {}",
            crate::structural_lint::architecture::Settings::describe()
        ));
    }
    console::stdout(format!(
        "Use lint-rule {kind} --example for a complete configuration example."
    ));
}
