// DECISION: D020
mod hooks;
mod scaffold;

use agentrig::{diagnostics, lint, util};
use anyhow::{Result, bail};
use serde_json::Value;
use std::{
    env,
    io::{self, Read},
    path::{Path, PathBuf},
};
use util::take_option;

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
        "review" => run_review(&root, args),
        "delegate" => run_delegate(&root, args),
        "lint" | "lint-config-check" | "lint-explain" => run_lint(&root, &mut args, &command),
        "lint-rules" | "lint-rule" => lint::cli::discovery(&command, &args),
        _ => bail!(
            "usage: agentrig hook|lint|lint-config-check|lint-rules|lint-rule|guard-commit|guard-reference [--root PATH]"
        ),
    }
}
fn immediate(command: &str) -> Option<Result<i32>> {
    Some(match command {
        "review-hook" => review_runner::execution::broker::hook().map(|()| 0),
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
    if initializing {
        std::fs::create_dir_all(&root)?;
    }
    root.canonicalize().map_err(Into::into)
}
fn print_help() {
    println!(
        "delegate config-check CONFIG | mcp CONFIG | start CONFIG REQUEST | status RUN_ID | result RUN_ID | cancel RUN_ID\njobs | job-status RUN_ID | job-logs RUN_ID | job-start COMMAND | job-stop RUN_ID | job-cleanup [--branch BRANCH]"
    );
    println!(
        "agentrig (Linux)\nreview config-check CONFIG | review run CONFIG REQUEST_JSON | review mcp CONFIG\nupgrade plan RELEASE_EXECUTABLE | upgrade apply PLAN | upgrade rollback\ninit | setup | doctor | config-check | commands | run NAME [-- ARGS] | report\ncheck [--staged] [--only CHECK_ID] | memory-check | resume | feature-start NAME | feature-merge\nhook | lint | lint-config-check | lint-rules | lint-rule ID [--json|--example] | lint-explain PATH [--json] | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --language python|rust, --source, --memory, --skills, --service, --base, --prefix and --review true|false."
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
fn run_review(root: &Path, mut args: Vec<String>) -> Result<i32> {
    let configured = matches!(args.as_slice(), [command] if matches!(command.as_str(), "mcp" | "config-check"))
        || matches!(args.as_slice(), [command, _] if command == "run");
    if configured {
        let context = scaffold::config::Context::load(root)?;
        let review = context.config.capabilities.review.as_ref().ok_or_else(|| {
            anyhow::anyhow!("review is not enabled; configure capabilities.review.config")
        })?;
        args.insert(
            1,
            context.path(&review.config)?.to_string_lossy().into_owned(),
        );
    }
    for argument in args.iter_mut().skip(1) {
        *argument = root.join(&*argument).to_string_lossy().into_owned();
    }
    review_runner::cli::run(&args).map(|()| 0)
}
fn run_delegate(root: &Path, mut args: Vec<String>) -> Result<i32> {
    let configured = matches!(args.as_slice(), [command] if matches!(command.as_str(), "mcp" | "config-check"))
        || matches!(args.as_slice(), [command, _] if command == "start");
    if configured {
        let context = scaffold::config::Context::load(root)?;
        let delegation = context
            .config
            .capabilities
            .delegation
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "delegation is not enabled; configure capabilities.delegation.config"
                )
            })?;
        args.insert(
            1,
            context
                .path(&delegation.config)?
                .to_string_lossy()
                .into_owned(),
        );
    }
    let direct = args
        .first()
        .is_some_and(|command| matches!(command.as_str(), "config-check" | "_execute"));
    if direct {
        return agentrig::delegate::cli(root, &args);
    }
    let context = scaffold::config::Context::load(root)?;
    agentrig::delegate::run::cli(root, &context.path(&context.config.paths.runtime)?, &args)
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
