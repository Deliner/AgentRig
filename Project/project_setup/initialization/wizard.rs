use crate::project_config;
use crate::project_setup::template::{self, Scaffold};
use crate::project_setup::{self as package, Config, Files};
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
    super::no_overwrite_init::prepare(&root, config, &files)?;
    console::stdout(format!(
        "Configuration for {root}:\n{config}\nFiles:\n{files}",
        root = root.display(),
        config = crate::shared::strict_yaml::encode(config)?,
        files = listing(&root, &scaffold, &files)
    ));
    let Some(answer) = ask("Install this environment? yes/no", "no".into())? else {
        return cancelled();
    };
    let confirmed = matches!(answer.to_ascii_lowercase().as_str(), "yes" | "y");
    if confirmed {
        super::no_overwrite_init::prepare(&root, config, &files)?;
        crate::shared::fs_with_context::create_dir_all(&root)?;
        super::no_overwrite_init::install(&root, &scaffold, &files)?;
        console::stdout("Installed the worker environment. Authentication remains separate.");
        super::no_overwrite_init::offer_ratchet(&root, config)?;
        package::doctor(&project_config::Context::load(&root)?)
    } else {
        cancelled()
    }
}

fn listing(root: &Path, scaffold: &Scaffold, files: &Files) -> String {
    let clippy = scaffold
        .clippy
        .iter()
        .map(|settings| settings.path().trim_prefix(root).display().to_string());
    let listed: Vec<String> = files.keys().cloned().chain(clippy).collect();
    listed.join("\n")
}

fn selection(root: &Path) -> Result<Option<(PathBuf, Scaffold)>> {
    let Some(target) = ask("Project directory", root.to_string_lossy().into_owned())? else {
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
    let available = check_names(&scaffold.config);
    let label = format!("Checks, comma-separated: {available}");
    let Some(checks) = ask(&label, available)? else {
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
        let Some(value) = ask(label, default)? else {
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
            "unknown check {name}; supported: {}",
            check_names(config)
        );
    }
    config
        .checks
        .retain(|check| unique.contains(check.id.as_str()));
    Ok(())
}

fn check_names(config: &Config) -> String {
    let names: Vec<&str> = config
        .checks
        .iter()
        .map(|check| check.id.as_str())
        .collect();
    names.join(",")
}

fn ask(label: &str, default: String) -> Result<Option<String>> {
    console::prompt(format!("{label} [{default}]: "))?;
    let mut line = String::new();
    let read = io::stdin().read_line(&mut line)?;
    let cancelled = read == 0 || line.trim().eq_ignore_ascii_case("cancel");
    if cancelled {
        return Ok(None);
    }
    let answer = line.trim();
    let empty = answer.is_empty();
    Ok(Some(if empty { default } else { answer.to_owned() }))
}

fn cancelled() -> Result<i32> {
    console::stdout("Initialization cancelled; no environment installed.");
    Ok(0)
}
