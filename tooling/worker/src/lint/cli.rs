use crate::util::take_option;
use anyhow::{Result, ensure};
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn execute(root: &Path, config: &Path, args: &[String], validate_only: bool) -> Result<i32> {
    ensure!(
        args.iter().all(|arg| arg == "--json"),
        "unknown lint argument"
    );
    let json = args.iter().any(|arg| arg == "--json");
    super::run(root, config, json, validate_only)
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
    let rules = command == "lint-rules";
    if rules {
        ensure!(args.is_empty(), "lint-rules takes no arguments");
        println!("{}", super::rules::catalog());
        return Ok(0);
    }
    ensure!(
        matches!(command.as_str(), "lint" | "lint-config-check"),
        "expected lint, lint-config-check or lint-rules"
    );
    let root = take_option(&mut args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?)
        .canonicalize()?;
    let config = take_option(&mut args, "--config")?.unwrap_or_else(|| "lint.toml".into());
    execute(
        &root,
        &root.join(config),
        &args,
        command == "lint-config-check",
    )
}
