use super::{Config, config, setup, template};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeSet,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub fn run(root: &Path) -> Result<i32> {
    println!(
        "AgentRig setup. Enter accepts the displayed default; 'cancel' or EOF cancels before installation."
    );
    let Some((root, config)) = selection(root)? else {
        return cancelled();
    };
    let installation = setup::prepare(&root, &config, super::bundle(&root, &config)?)?;
    println!(
        "Configuration for {}:\n{}",
        root.display(),
        review_runner::config::yaml::encode(&config)?
    );
    setup::print_preview(&root, &config, &installation)?;
    let Some(answer) = ask("Install this environment? yes/no", "no")? else {
        return cancelled();
    };
    let confirmed = matches!(answer.to_ascii_lowercase().as_str(), "yes" | "y");
    if confirmed {
        std::fs::create_dir_all(&root)?;
        setup::install(&root, &config, &installation)
    } else {
        cancelled()
    }
}

fn selection(root: &Path) -> Result<Option<(PathBuf, Config)>> {
    let Some(target) = ask("Project directory", &root.to_string_lossy())? else {
        return Ok(None);
    };
    let root = crate::paths::resolve(&std::env::current_dir()?.join(target))?;
    ensure!(
        !root.join(config::FILE).exists(),
        "project already has agentrig.yaml; use setup or upgrade"
    );
    let Some(mut config) = project_config(&root)? else {
        return Ok(None);
    };
    let Some(delegation) = ask(
        "Delegate profiles YAML, relative to project; empty disables",
        "",
    )?
    else {
        return Ok(None);
    };
    let enabled = !delegation.is_empty();
    if enabled {
        config.capabilities.delegation = Some(config::Resource { config: delegation });
    }
    let Some(checks) = ask(
        "Checks, comma-separated: lint,memory,tests",
        "lint,memory,tests",
    )?
    else {
        return Ok(None);
    };
    select_checks(&mut config, &checks)?;
    Ok(Some((root, config)))
}

fn project_config(root: &Path) -> Result<Option<Config>> {
    let mut options = template::options(root, &[])?;
    for (key, label) in [
        ("language", "Language: python/rust"),
        ("service", "Service directory"),
        ("source", "Source directory"),
        ("memory", "Memory directory"),
        ("skills", "Skills directory"),
        ("base", "Base branch"),
        ("prefix", "Feature branch prefix"),
        ("review", "Enable review: true/false"),
        ("vcs", "Version control: git/mercurial"),
        ("frontend", "Agent client: codex/claude-code"),
    ] {
        let default = match key {
            "skills" => format!("{}/skills", options["service"]),
            _ => options[key].clone(),
        };
        let Some(value) = ask(label, &default)? else {
            return Ok(None);
        };
        options.insert(key, value);
    }
    let args: Vec<String> = options
        .iter()
        .flat_map(|(key, value)| [format!("--{key}"), value.clone()])
        .collect();
    Ok(Some(template::config(&template::options(root, &args)?)))
}

fn select_checks(config: &mut Config, value: &str) -> Result<()> {
    let selected: Vec<_> = value.split(',').map(str::trim).collect();
    let unique: BTreeSet<_> = selected.iter().copied().collect();
    ensure!(unique.len() == selected.len(), "duplicate selected check");
    for name in &selected {
        ensure!(
            config.checks.iter().any(|check| check.id == *name),
            "unknown check {name}; supported: lint,memory,tests"
        );
    }
    config
        .checks
        .retain(|check| unique.contains(check.id.as_str()));
    Ok(())
}

fn ask(label: &str, default: &str) -> Result<Option<String>> {
    print!("{label} [{default}]: ");
    io::stdout().flush()?;
    let mut line = String::new();
    let read = io::stdin().read_line(&mut line)?;
    let cancelled = read == 0 || line.trim().eq_ignore_ascii_case("cancel");
    if cancelled {
        return Ok(None);
    }
    let answer = line.trim();
    let empty = answer.is_empty();
    Ok(Some(if empty {
        default.to_owned()
    } else {
        answer.to_owned()
    }))
}

fn cancelled() -> Result<i32> {
    println!("Initialization cancelled; no environment installed.");
    Ok(0)
}
