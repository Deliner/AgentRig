// DECISION: D020
mod hooks;
mod lint;
mod scaffold;
mod util;

use anyhow::{Result, bail};
use serde_json::Value;
use std::{
    env,
    io::{self, Read},
    path::PathBuf,
};

// DECISION: D015
// DECISION: D016
// DECISION: D018
fn run() -> Result<i32> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let command = if args.is_empty() {
        String::new()
    } else {
        args.remove(0)
    };
    if command == "--version" {
        println!("discipline-worker {}", scaffold::config::VERSION);
        return Ok(0);
    }
    if command == "--help" {
        println!(
            "discipline-worker (Linux)\ninit | doctor | config-check | commands | run NAME [-- ARGS] | report\ncheck [--staged] | memory-check | resume | feature-start NAME | feature-merge\nhook | lint | lint-config-check | lint-rules | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --language python|rust, --source, --memory, --skills, --base and --prefix."
        );
        return Ok(0);
    }
    let root = take_option(&mut args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    if command == "init" {
        std::fs::create_dir_all(&root)?;
    }
    let root = root.canonicalize()?;
    if scaffold::owns(&command) || matches!(command.as_str(), "guard-commit" | "guard-reference") {
        return scaffold::run(&root, &command, &args);
    }
    match command.as_str() {
        "hook" => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let result = serde_json::from_str::<Value>(&input)
                .map_err(anyhow::Error::from)
                .and_then(|event| hooks::dispatch(&root, &event));
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
        "lint" | "lint-config-check" => {
            let explicit = take_option(&mut args, "--config")?;
            let config = match explicit {
                Some(path) => root.join(path),
                None if root.join(scaffold::config::FILE).is_file() => {
                    let context = scaffold::config::Context::load(&root)?;
                    context.path(&context.config.paths.lint)?
                }
                None => root.join("lint.toml"),
            };
            let json = args.iter().any(|arg| arg == "--json");
            if args.iter().any(|arg| arg != "--json") {
                bail!("unknown lint argument");
            }
            lint::run(&root, &config, json, command == "lint-config-check")
        }
        "lint-rules" => {
            println!("{}", lint::rules::catalog());
            Ok(0)
        }
        _ => bail!(
            "usage: discipline-worker hook|lint|lint-config-check|lint-rules|guard-commit|guard-reference [--root PATH]"
        ),
    }
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
    if index < args.len() && !args[index].starts_with("--") {
        Ok(Some(args.remove(index)))
    } else {
        bail!("{name} requires a value")
    }
}
fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("worker: {error:#}");
            2
        }
    };
    std::process::exit(code);
}
