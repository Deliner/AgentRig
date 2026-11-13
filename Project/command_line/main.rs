// DECISION: D020

use agentrig::shared::console;
use agentrig::shared::diagnostics;
use agentrig::shared::shared_options::take_option;
use agentrig::structural_lint::Location;
use agentrig::{agent_hooks, command_line, project_config, structural_lint};
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
    if let Some(workflow) = command_line::Command::parse(&command) {
        return command_line::run(&root, workflow, &args);
    }
    match command.as_str() {
        "hook" => hook(&root),
        "lint" | "lint-config-check" | "lint-explain" | "lint-edges" | "lint-tree"
        | "lint-baseline" => run_lint(&root, &mut args, &command),
        "lint-rules" | "lint-rule" => structural_lint::reporting::cli::discovery(&command, &args),
        "lint-structure" => structural_lint::reporting::tree::lint_structure(&args),
        _ => bail!("unknown command {command:?}; run agentrig --help for the command table"),
    }
}
fn immediate(command: &str) -> Option<Result<i32>> {
    Some(match command {
        "--version" => {
            console::stdout(format!("agentrig {}", project_config::VERSION));
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
    let root = match take_option(args, "--root")? {
        Some(value) => PathBuf::from(value),
        None => env::current_dir()?,
    };
    let initializing = command == "init";
    let interactive = initializing && args.iter().any(|arg| arg == "--interactive");
    if interactive {
        return agentrig::shared::paths::resolve(&env::current_dir()?.join(root));
    }
    if initializing {
        agentrig::shared::fs_with_context::create_dir_all(&root)?;
    }
    Ok(agentrig::shared::fs_with_context::canonicalize(&root)?)
}
fn print_help() {
    console::stdout(
        "agentrig\ninit [--interactive] | runtime-add | doctor | config-check | commands | run NAME [-- ARGS] | scratch -- TEXT | report\ncheck [--staged | --revision REV] [--only CHECK_ID] | memory-check | resume | feature-start NAME | feature-merge\njobs | job-status RUN_ID | job-logs RUN_ID | job-start COMMAND | job-stop RUN_ID | job-cleanup [--branch BRANCH]\nhook | lint | lint-config-check | lint-baseline | lint-rules | lint-rule ID [--json|--example] | lint-explain PATH [--json] | lint-edges PATH [--json] | lint-tree PATH | lint-structure PATH | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --language python|rust, --frontend codex|claude-code, --source, --memory, --skills, --service, --base and --prefix.",
    );
}
fn hook(root: &Path) -> Result<i32> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let result = serde_json::from_str::<Value>(&input)
        .map_err(anyhow::Error::from)
        .and_then(|event| agent_hooks::dispatch(root, &event));
    let output = match result {
        Ok(value) => value,
        Err(error) => Some(agent_hooks::deny(&format!(
            "Worker hook could not process this event: {error}"
        ))),
    };
    if let Some(value) = output {
        console::stdout(value);
    }
    Ok(0)
}
fn run_lint(root: &Path, args: &mut Vec<String>, command: &str) -> Result<i32> {
    let explicit = take_option(args, "--config")?;
    let resolved = match explicit {
        Some(path) => Ok(root.join(path)),
        None if root.join(project_config::FILE).is_file() => project_lint(root),
        None => Ok(root.join("lint.yaml")),
    };
    // DECISION: D143
    let inspection = structural_lint::reporting::explain::inspection(command);
    // DECISION: D123
    let baseline = command == "lint-baseline";
    anyhow::ensure!(
        !baseline || args.is_empty(),
        "lint-baseline takes only --root and --config"
    );
    let reported = inspection.is_none() && !baseline;
    let config = match resolved {
        Ok(config) => config,
        Err(error) if reported => return unresolved(root, command, args, error),
        Err(error) => return Err(error),
    };
    let location = Location {
        root,
        config: &config,
    };
    if let Some(inspect) = inspection {
        return inspect(location, args);
    }
    if baseline {
        return structural_lint::baseline::record(location);
    }
    structural_lint::reporting::cli::execute(location, args, structural_lint::Mode::of(command))
}
fn unresolved(root: &Path, command: &str, args: &[String], error: anyhow::Error) -> Result<i32> {
    let invocation: Vec<String> = std::iter::once(command.to_owned())
        .chain(args.iter().cloned())
        .collect();
    let path = root.join(project_config::FILE);
    let location = Location {
        root,
        config: &path,
    };
    structural_lint::reporting::cli::failure(location, error, &invocation)
}
fn project_lint(root: &Path) -> Result<PathBuf> {
    let context = project_config::Context::load(root)?;
    anyhow::ensure!(
        context.config.capabilities.lint,
        "capabilities.lint is disabled"
    );
    context.path(&context.config.paths.lint)
}
fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            console::stderr(diagnostics::failure(&error));
            2
        }
    };
    std::process::exit(code);
}
