// DECISION: D020

use agentrig::cli_dispatch::{commands::Command, dispatch};
use agentrig::shared::calling_agent::{Agent, Caller};
use agentrig::shared::console;
use agentrig::shared::diagnostics;
use agentrig::shared::shared_options::take_option;
use agentrig::structural_lint::Location;
use agentrig::{agent_hooks, command_runner, project_config, structural_lint};
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
    if let Some(workflow) = Command::parse(&command) {
        return dispatch::run(&root, workflow, &args);
    }
    match command.as_str() {
        "hook" => hook(&root, &mut args),
        "lint" | "lint-config-check" | "lint-explain" | "lint-edges" | "lint-tree" => {
            run_lint(&root, &mut args, &command)
        }
        "lint-rules" | "lint-rule" => structural_lint::discovery(&command, &args),
        "lint-structure" => structural_lint::lint_structure(&args),
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
        "agentrig\ninit [--interactive] | runtime-add | doctor | config-check | commands | run NAME [-- ARGS] | report\ncheck [--staged | --revision REV] [--only CHECK_ID] | memory-check | resume | feature-start NAME | feature-merge | feature-push | state-reset | approve | propose-pin REVISION\njobs | job-status RUN_ID | job-logs RUN_ID | job-start COMMAND | job-stop RUN_ID | job-cleanup [--branch BRANCH]\nhook --agent codex|claude-code | lint | lint-config-check | lint-rules | lint-rule ID [--json|--example] | lint-explain PATH [--json] | lint-edges PATH [--json] | lint-tree PATH | lint-structure PATH | guard-commit | guard-reference\nUse --root PATH to select the project. init accepts --key KEY, --language python|rust, --skills, --service, --base and --prefix.",
    );
}
// DECISION: D188
fn hook(root: &Path, args: &mut Vec<String>) -> Result<i32> {
    let names = Agent::ALL.map(Agent::name).join("|");
    let name = take_option(args, "--agent")?
        .ok_or_else(|| anyhow::anyhow!("hook requires --agent {names} from its registration"))?;
    let agent = Agent::parse(&name)?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let result = serde_json::from_str::<Value>(&input)
        .map_err(anyhow::Error::from)
        .and_then(|event| agent_hooks::dispatch(root, &event, agent));
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
struct Settings {
    lint: PathBuf,
    rust: Option<RustProject>,
}
// DECISION: D239
struct RustProject {
    context: project_config::Context,
    manifest: PathBuf,
}
impl RustProject {
    fn of(context: project_config::Context) -> Result<Option<Self>> {
        let manifest = context.manifest()?;
        Ok(manifest.map(|manifest| Self { context, manifest }))
    }
}
fn run_lint(root: &Path, args: &mut Vec<String>, command: &str) -> Result<i32> {
    let explicit = take_option(args, "--config")?;
    // DECISION: D143
    let inspection = structural_lint::inspection(command);
    let reported = inspection.is_none();
    // DECISION: D188
    let caller = diagnostics::caller(root, Agent::calling());
    let settings = match settings(root, explicit, command) {
        Ok(settings) => settings,
        Err(error) if reported => return unresolved(&caller, command, args, error),
        Err(error) => return Err(error),
    };
    let location = Location {
        root,
        config: &settings.lint,
    };
    if let Some(inspect) = inspection {
        return inspect(location, args);
    }
    let Some(rust) = &settings.rust else {
        let mode = structural_lint::Mode::of(command);
        return structural_lint::execute(location, &caller, args, mode);
    };
    let launch = |argv: &[String]| command_runner::output(&rust.context, argv);
    let clippy = Some(structural_lint::Clippy {
        manifest: &rust.manifest,
        skills: Path::new(&rust.context.config.paths.skills),
        launch: &launch,
    });
    let mode = structural_lint::Mode::Lint { clippy };
    structural_lint::execute(location, &caller, args, mode)
}
fn settings(root: &Path, explicit: Option<String>, command: &str) -> Result<Settings> {
    let configured = root.join(project_config::FILE).is_file();
    let linted = command == "lint";
    let needed = explicit.is_none() || linted;
    let loaded = configured && needed;
    let context = loaded
        .then(|| project_config::Context::load(root))
        .transpose()?;
    let lint = match (explicit, &context) {
        (Some(path), _) => root.join(path),
        (None, Some(context)) => project_lint(context)?,
        (None, None) => root.join("lint.yaml"),
    };
    let rust = match context {
        Some(context) if linted => RustProject::of(context)?,
        Some(_) | None => None,
    };
    Ok(Settings { lint, rust })
}
fn unresolved(
    caller: &Caller,
    command: &str,
    args: &[String],
    error: anyhow::Error,
) -> Result<i32> {
    let invocation: Vec<String> = std::iter::once(command.to_owned())
        .chain(args.iter().cloned())
        .collect();
    let path = caller.root.join(project_config::FILE);
    let location = Location {
        root: &caller.root,
        config: &path,
    };
    structural_lint::failure(location, caller, error, &invocation)
}
fn project_lint(context: &project_config::Context) -> Result<PathBuf> {
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
