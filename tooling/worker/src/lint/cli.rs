use crate::arguments::take_option;
use anyhow::{Result, ensure};
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn execute(root: &Path, config: &Path, args: &[String], validate_only: bool) -> Result<i32> {
    let mut args = args.to_vec();
    let selected = selection(root, &mut args)?;
    ensure!(
        args.iter().all(|arg| arg == "--json"),
        "unknown lint argument"
    );
    let source = selected
        .as_ref()
        .map(|(_, backend)| backend.repository_source(root))
        .transpose()?
        .flatten();
    let command = if validate_only {
        "lint-config-check"
    } else {
        "lint"
    };
    let rerun = selected.as_ref().map(|(path, _)| {
        crate::diagnostics::rerun(
            root,
            &[
                command.into(),
                "--config".into(),
                config.to_string_lossy().into_owned(),
                "--vcs-config".into(),
                path.to_string_lossy().into_owned(),
            ],
        )
    });
    super::run_output(
        root,
        config,
        super::Output {
            json: args.iter().any(|arg| arg == "--json"),
            validate_only,
            rerun: rerun.as_deref(),
            source: source.as_ref(),
        },
    )
}

pub(super) fn selection(
    root: &Path,
    args: &mut Vec<String>,
) -> Result<Option<(PathBuf, review_runner::vcs::Backend)>> {
    let Some(path) = take_option(args, "--vcs-config")? else {
        return Ok(None);
    };
    let path = root.join(path).canonicalize()?;
    let backend: review_runner::vcs::Backend = review_runner::config::yaml::read(&path)?;
    backend.validate()?;
    Ok(Some((path, backend)))
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
            "lint" | "lint-config-check" | "lint-explain"
        ),
        "expected lint, lint-config-check, lint-explain, lint-rule or lint-rules"
    );
    let root = take_option(&mut args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?)
        .canonicalize()?;
    let config = take_option(&mut args, "--config")?.unwrap_or_else(|| "lint.yaml".into());
    let explain = command == "lint-explain";
    if explain {
        return super::explain::run(&root, &root.join(config), &args);
    }
    execute(
        &root,
        &root.join(config),
        &args,
        command == "lint-config-check",
    )
}

pub fn discovery(command: &str, args: &[String]) -> Result<i32> {
    let catalog = command == "lint-rules";
    if catalog {
        ensure!(args.is_empty(), "lint-rules takes no arguments");
        println!("{}", super::rules::catalog());
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
            "rules": [super::rules::example(kind, ".agents/skills", &["src/**".into()])]});
        println!("{}", review_runner::config::yaml::encode(&value)?);
    } else if json {
        println!("{}", super::rules::describe(kind));
    } else {
        print_rule(kind);
    }
    Ok(0)
}

fn print_rule(kind: super::rules::Kind) {
    let descriptor = kind.descriptor();
    println!(
        "{kind}: {}\nTarget: {}\nRepair skill: {}\nParameters: {}",
        descriptor.metric,
        descriptor.target,
        descriptor.skill,
        descriptor.parameters.describe()
    );
    let syntax = super::rules::syntax(kind);
    if syntax {
        println!("Languages: {}", super::rules::support(kind));
    }
    let architecture = kind == super::rules::Kind::DirectoryArchitecture;
    if architecture {
        println!(
            "Architecture settings and limits: {}",
            super::architecture::Settings::describe()
        );
    }
    println!("Use lint-rule {kind} --example for a complete configuration example.");
}
