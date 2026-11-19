// DECISION: D005
use crate::change_delivery::gate;
use crate::command_runner;
use crate::project_config::Context;
use crate::project_setup;
use crate::shared::paths::resolve;
use crate::shared::shared_options::forwarded;
use anyhow::{Result, bail, ensure};
use serde_json::Value;
use std::path::{Path, PathBuf};

// DECISION: D196
const PATH_PROGRAMS: [&str; 6] = ["mkdir", "cp", "mv", "rm", "touch", "chmod"];
// DECISION: D201
const QUOTED_SUBSTITUTIONS: [&[char]; 3] = [&['\u{60}'], &['$', '('], &['$', '{', '(']];
const LISTING: project_setup::Recipe = project_setup::Recipe {
    command: "--list",
    forwards: false,
};

enum Role {
    StartEscape,
    InQuote(char),
    OpenQuote,
    Control,
    Plain,
}
// DECISION: D201
fn role(quote: Option<char>, ch: char) -> Role {
    let escape_start = ch == '\\' && quote != Some('\'');
    let quote_start = ch == '\'' || ch == '"';
    let shell_control = ";|&<>(\n\u{60}".contains(ch);
    if escape_start {
        Role::StartEscape
    } else if let Some(q) = quote {
        Role::InQuote(q)
    } else if quote_start {
        Role::OpenQuote
    } else if shell_control {
        Role::Control
    } else {
        Role::Plain
    }
}
// DECISION: D015
fn outer_control(command: &str) -> bool {
    let chars: Vec<char> = command.chars().collect();
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().copied().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match role(quote, ch) {
            Role::StartEscape => escaped = true,
            Role::InQuote(q) => {
                let quote_end = ch == q;
                let rest = &chars[index..];
                let substitutes = QUOTED_SUBSTITUTIONS
                    .iter()
                    .any(|opening| rest.starts_with(opening));
                let quoted_substitution = q == '"' && substitutes;
                if quote_end {
                    quote = None;
                } else if quoted_substitution {
                    return true;
                }
            }
            Role::OpenQuote => quote = Some(ch),
            Role::Control => return true,
            Role::Plain => {}
        }
    }
    quote.is_some() || escaped
}
pub fn command(event: &Value) -> Option<&str> {
    ["cmd", "command"]
        .iter()
        .find_map(|key| event["tool_input"][key].as_str())
}
pub fn words(command: &str) -> Result<Vec<String>> {
    let has_outer_control = outer_control(command);
    if has_outer_control {
        bail!(
            "Agent shell commands must be one top-level just invocation; quote any character the shell would read as control, substitution or a glob qualifier."
        );
    }
    Ok(shell_words::split(command)?)
}
// DECISION: D201
pub fn arguments(command: Option<&str>) -> Result<Vec<String>> {
    let command =
        command.ok_or_else(|| anyhow::anyhow!("one top-level just invocation is required"))?;
    let argv = words(command)?;
    let named_just = argv.first().is_some_and(|program| program == "just");
    ensure!(
        named_just,
        "Direct shell commands are disabled; run a recipe from just --list by the name just, not by a path."
    );
    Ok(argv)
}

// DECISION: D047
fn enters_check(argv: &[String]) -> Result<bool> {
    let recipe = argv.get(1).is_some_and(|arg| arg == "check");
    let mut words = Vec::new();
    for word in argv {
        let split = shell_words::split(word)
            .map_err(|error| anyhow::anyhow!("cannot analyze shell word {word:?}: {error}"))?;
        words.extend(split);
    }
    let git = words.iter().position(|word| {
        Path::new(word)
            .file_name()
            .is_some_and(|name| name == "git")
    });
    let commit = git.is_some_and(|start| words[start..].iter().any(|word| word == "commit"));
    Ok(recipe || commit)
}

pub struct Inspection {
    pub catalog: Option<String>,
    pub check: bool,
    pub written: Option<Vec<PathBuf>>,
}
enum Call<'a> {
    Catalog {
        name: &'a str,
        args: &'a [String],
    },
    Runtime {
        command: &'static str,
        args: &'a [String],
    },
}

// DECISION: D201
pub fn inspect(context: &Context, argv: &[String]) -> Result<Inspection> {
    let catalog = match call(argv)? {
        Call::Catalog { name, args } => Some((name, command_runner::argv(context, name, args)?)),
        Call::Runtime { command, args } => {
            runtime_arguments(command, args)?;
            None
        }
    };
    let written = catalog
        .as_ref()
        .and_then(|(_, command)| written(&context.root, command));
    Ok(Inspection {
        catalog: catalog.map(|(name, _)| name.to_owned()),
        check: enters_check(argv)?,
        written,
    })
}
// DECISION: D197
fn call(argv: &[String]) -> Result<Call<'_>> {
    let (name, extra) = argv
        .get(1..)
        .and_then(|words| words.split_first())
        .ok_or_else(|| anyhow::anyhow!("name one recipe from just --list"))?;
    let listing = name == "--list";
    let recipe = if listing {
        Some(LISTING)
    } else {
        project_setup::recipe(name)
    };
    let Some(recipe) = recipe else {
        return Ok(Call::Catalog {
            name,
            args: forwarded(extra),
        });
    };
    ensure!(
        recipe.forwards || extra.is_empty(),
        "one Just recipe is required; `just {name}` takes no arguments, and Just would run a further word as another recipe"
    );
    recipe_call(recipe.command, extra)
}
fn recipe_call<'a>(command: &'static str, extra: &'a [String]) -> Result<Call<'a>> {
    match command {
        "run" | "job-start" => {
            let (name, args) = extra
                .split_first()
                .ok_or_else(|| anyhow::anyhow!("just {command} requires a catalog command"))?;
            Ok(Call::Catalog {
                name,
                args: forwarded(args),
            })
        }
        _ => match command.strip_prefix("run ") {
            Some(name) => Ok(Call::Catalog {
                name,
                args: forwarded(extra),
            }),
            None => Ok(Call::Runtime {
                command,
                args: extra,
            }),
        },
    }
}
fn runtime_arguments(command: &str, args: &[String]) -> Result<()> {
    match command {
        "check" => {
            gate::arguments(forwarded(args))?;
        }
        "feature-start" => {
            ensure!(forwarded(args).len() == 1, "feature-start NAME");
        }
        _ => {}
    }
    Ok(())
}

// DECISION: D196
fn written(root: &Path, command: &[String]) -> Option<Vec<PathBuf>> {
    let (program, words) = command.split_first()?;
    let path_program = PATH_PROGRAMS.contains(&program.as_str());
    if path_program {
        operands(words).and_then(|operands| rooted(root, &operands))
    } else {
        None
    }
}
fn operands(words: &[String]) -> Option<Vec<String>> {
    let mut operands = Vec::new();
    let mut options = true;
    for word in words {
        let separator = options && word == "--";
        let option = options && word.starts_with('-');
        if separator {
            options = false;
        } else if option {
            let carries_value = word.contains(['=', '/']);
            if carries_value {
                return None;
            }
        } else {
            operands.push(word.clone());
        }
    }
    Some(operands)
}
fn rooted(root: &Path, operands: &[String]) -> Option<Vec<PathBuf>> {
    let resolved: Option<Vec<PathBuf>> = operands
        .iter()
        .map(|operand| resolve(&root.join(operand)).ok())
        .collect();
    let inside = resolved?
        .into_iter()
        .filter_map(|path| path.strip_prefix(root).ok().map(Path::to_owned));
    Some(inside.collect())
}
