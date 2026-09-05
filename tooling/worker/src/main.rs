// DECISION: D020
mod diagnostics;
mod hooks;
mod lint;
mod scaffold;
mod util;

use anyhow::{Result, bail};
use serde_json::Value;
use std::{
    env,
    io::{self, Read},
    path::{Path, PathBuf},
};

// DECISION: D015
// DECISION: D016
// DECISION: D018
fn run() -> Result<i32> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().cloned().unwrap_or_default();
    let has_command = !args.is_empty();
    if has_command {
        args.remove(0);
    }
    match command.as_str() {
        "--version" => {
            println!("discipline-worker {}", scaffold::config::VERSION);
            return Ok(0);
        }
        "--help" => {
            print_help();
            return Ok(0);
        }
        _ => {}
    }
    let root = project_root(&mut args, &command)?;
    let scaffold_command =
        scaffold::owns(&command) || matches!(command.as_str(), "guard-commit" | "guard-reference");
    if scaffold_command {
        return scaffold::run(&root, &command, &args);
    }
    match command.as_str() {
        "hook" => hook(&root),
        "lint" | "lint-config-check" => run_lint(&root, &mut args, command == "lint-config-check"),
        "lint-rules" => {
            println!("{}", lint::rules::catalog());
            Ok(0)
        }
        _ => bail!(
            "usage: discipline-worker hook|lint|lint-config-check|lint-rules|guard-commit|guard-reference [--root PATH]"
        ),
    }
}
fn project_root(args: &mut Vec<String>, command: &str) -> Result<PathBuf> {
    let root = take_option(args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let initializing = command == "init";
    if initializing {
        std::fs::create_dir_all(&root)?;
    }
    root.canonicalize().map_err(Into::into)
}
fn print_help() {
    println!(
        "discipline-worker (Linux)\ninit | doctor | config-check | commands | run NAME [-- ARGS] | report\ncheck [--staged] [--only CHECK_ID] | memory-check | resume | feature-start NAME | feature-merge\nhook | lint | lint-config-check | lint-rules | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --language python|rust, --source, --memory, --skills, --base and --prefix."
    );
}
fn hook(root: &Path) -> Result<i32> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let result = serde_json::from_str::<Value>(&input)
        .map_err(anyhow::Error::from)
        .and_then(|event| hooks::dispatch(root, &event));
    let output = match result {
        Ok(value) => value,
        Err(error) => Some(hooks::deny(&format!(
            "Worker hook could not process this event: {error}"
        ))),
    };
    if let Some(value) = output {
        println!("{value}");
    }
    Ok(0)
}
fn run_lint(root: &Path, args: &mut Vec<String>, validate_only: bool) -> Result<i32> {
    let explicit = take_option(args, "--config")?;
    let config = match explicit {
        Some(path) => root.join(path),
        None if root.join(scaffold::config::FILE).is_file() => {
            let context = scaffold::config::Context::load(root)?;
            context.path(&context.config.paths.lint)?
        }
        None => root.join("lint.toml"),
    };
    let json = args.iter().any(|arg| arg == "--json");
    let unknown_argument = args.iter().any(|arg| arg != "--json");
    if unknown_argument {
        bail!("unknown lint argument");
    }
    lint::run(root, &config, json, validate_only)
}
fn take_option(args: &mut Vec<String>, name: &str) -> Result<Option<String>> {
    let Some(index) = args
        .iter()
        .take_while(|arg| arg.as_str() != "--")
        .position(|arg| arg == name)
    else {
        return Ok(None);
    };
    args.remove(index);
    let has_value = index < args.len() && !args[index].starts_with("--");
    if has_value {
        Ok(Some(args.remove(index)))
    } else {
        bail!("{name} requires a value")
    }
}
fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{}", diagnostics::failure(&error));
            2
        }
    };
    std::process::exit(code);
}
