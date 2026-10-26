// DECISION: D020

use agentrig::arguments::take_option;
use agentrig::{diagnostics, hooks, lint, scaffold};
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
    if let Some(result) = immediate(&command) {
        return result;
    }
    let root = project_root(&mut args, &command)?;
    let scaffold_command =
        scaffold::owns(&command) || matches!(command.as_str(), "guard-commit" | "guard-reference");
    if scaffold_command {
        return scaffold::run(&root, &command, &args);
    }
    match command.as_str() {
        "hook" => hook(&root),
        "lint" | "lint-config-check" | "lint-explain" => run_lint(&root, &mut args, &command),
        "lint-rules" | "lint-rule" => lint::cli::discovery(&command, &args),
        _ => bail!(
            "usage: agentrig hook|lint|lint-config-check|lint-rules|lint-rule|guard-commit|guard-reference [--root PATH]"
        ),
    }
}
fn immediate(command: &str) -> Option<Result<i32>> {
    Some(match command {
        "--version" => {
            println!("agentrig {}", scaffold::config::VERSION);
            Ok(0)
        }
        "--help" => {
            print_help();
            Ok(0)
        }
        _ => return None,
    })
}
fn project_root(args: &mut Vec<String>, command: &str) -> Result<PathBuf> {
    let root = take_option(args, "--root")?
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let initializing = command == "init";
    let interactive = initializing && args.iter().any(|arg| arg == "--interactive");
    if interactive {
        return agentrig::paths::resolve(&env::current_dir()?.join(root));
    }
    if initializing {
        std::fs::create_dir_all(&root)?;
    }
    root.canonicalize().map_err(Into::into)
}
fn print_help() {
    println!(
        "agentrig\ninit [--interactive] | doctor | config-check | commands | run NAME [-- ARGS] | report\ncheck [--staged | --revision REV] [--only CHECK_ID] | memory-check | resume | feature-start NAME | feature-merge\njobs | job-status RUN_ID | job-logs RUN_ID | job-start COMMAND | job-stop RUN_ID | job-cleanup [--branch BRANCH]\nhook | lint | lint-config-check | lint-rules | lint-rule ID [--json|--example] | lint-explain PATH [--json] | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --language python|rust, --frontend codex|claude-code, --source, --memory, --skills, --service, --base and --prefix."
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
fn run_lint(root: &Path, args: &mut Vec<String>, command: &str) -> Result<i32> {
    let explicit = take_option(args, "--config")?;
    let config = match explicit {
        Some(path) => root.join(path),
        None if root.join(scaffold::config::FILE).is_file() => {
            let context = scaffold::config::Context::load(root)?;
            anyhow::ensure!(
                context.config.capabilities.lint,
                "capabilities.lint is disabled"
            );
            context.path(&context.config.paths.lint)?
        }
        None => root.join("lint.yaml"),
    };
    let explain = command == "lint-explain";
    if explain {
        return lint::explain::run(root, &config, args);
    }
    lint::cli::execute(root, &config, args, command == "lint-config-check")
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
