use crate::project_config;
use crate::project_setup::template::{self, Scaffold};
use crate::project_setup::{self as package, Config};
use crate::shared::console;
use anyhow::{Result, ensure};
use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};

pub fn run(root: &Path) -> Result<i32> {
    console::stdout(
        "AgentRig setup. Enter accepts the displayed default; 'cancel' or EOF cancels before installation.",
    );
    let Some((root, scaffold)) = selection(root)? else {
        return cancelled();
    };
    let config = &scaffold.config;
    let files = package::bundle(&scaffold)?;
    super::init::prepare(&root, config, &files)?;
    console::stdout(format!(
        "Configuration for {}:\n{}\nFiles:\n{}",
        root.display(),
        crate::shared::yaml::encode(config)?,
        files.keys().cloned().collect::<Vec<_>>().join("\n")
    ));
    let Some(answer) = ask("Install this environment? yes/no", "no")? else {
        return cancelled();
    };
    let confirmed = matches!(answer.to_ascii_lowercase().as_str(), "yes" | "y");
    if confirmed {
        super::init::prepare(&root, config, &files)?;
        crate::shared::fs::create_dir_all(&root)?;
        super::init::install(&root, config, &files)?;
        console::stdout("Installed the worker environment. Authentication remains separate.");
        super::init::offer_ratchet(&root, config)?;
        package::doctor(&project_config::Context::load(&root)?)
    } else {
        cancelled()
    }
}

fn selection(root: &Path) -> Result<Option<(PathBuf, Scaffold)>> {
    let Some(target) = ask("Project directory", &root.to_string_lossy())? else {
        return Ok(None);
    };
    let root = crate::shared::paths::resolve(&std::env::current_dir()?.join(target))?;
    ensure!(
        !root.join(project_config::FILE).exists(),
        "project already has {}",
        project_config::FILE
    );
    let Some(mut scaffold) = project_config(&root)? else {
        return Ok(None);
    };
    let Some(checks) = ask(
        "Checks, comma-separated: lint,memory,tests,tools",
        "lint,memory,tests,tools",
    )?
    else {
        return Ok(None);
    };
    select_checks(&mut scaffold.config, &checks)?;
    Ok(Some((root, scaffold)))
}

fn project_config(root: &Path) -> Result<Option<Scaffold>> {
    let mut options = template::options(root, &[])?;
    for (key, label) in [
        ("language", "Language: python/rust"),
        ("service", "Service directory"),
        ("source", "Source directory"),
        ("memory", "Memory directory"),
        ("skills", "Skills directory"),
        ("base", "Base branch"),
        ("prefix", "Feature branch prefix"),
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
    let options = template::options(root, &args)?;
    Ok(Some(template::scaffold(root, &options)?))
}

fn select_checks(config: &mut Config, value: &str) -> Result<()> {
    let selected: Vec<_> = value.split(',').map(str::trim).collect();
    let unique: BTreeSet<_> = selected.iter().copied().collect();
    ensure!(unique.len() == selected.len(), "duplicate selected check");
    for name in &selected {
        ensure!(
            config.checks.iter().any(|check| check.id == *name),
            "unknown check {name}; supported: lint,memory,tests,tools"
        );
    }
    config
        .checks
        .retain(|check| unique.contains(check.id.as_str()));
    Ok(())
}

fn ask(label: &str, default: &str) -> Result<Option<String>> {
    console::prompt(format!("{label} [{default}]: "))?;
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
    console::stdout("Initialization cancelled; no environment installed.");
    Ok(0)
}
