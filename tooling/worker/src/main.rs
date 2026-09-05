mod gate;
mod hooks;
mod lint;
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
    let root = take_option(&mut args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let root = root.canonicalize()?;
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
            let config =
                take_option(&mut args, "--config")?.unwrap_or("tooling/worker/lint.toml".into());
            let json = args.iter().any(|arg| arg == "--json");
            if args.iter().any(|arg| arg != "--json") {
                bail!("unknown lint argument");
            }
            lint::run(
                &root,
                &root.join(config),
                json,
                command == "lint-config-check",
            )
        }
        "gate" => gate::run(&root, &args),
        "lint-rules" => {
            println!("{}", lint::rules::catalog());
            Ok(0)
        }
        "guard-commit" => hooks::git::guard_commit(&root),
        "guard-reference" => {
            hooks::git::guard_reference(&root, args.first().map(String::as_str).unwrap_or(""))
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
