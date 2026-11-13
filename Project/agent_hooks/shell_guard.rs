// DECISION: D005
use crate::change_delivery::gate;
use crate::command_runner;
use crate::project_config::Context;
use crate::shared::shared_options::forwarded;
use anyhow::{Result, bail};
use serde_json::Value;
use std::path::Path;

enum Role {
    StartEscape,
    InQuote(char),
    OpenQuote,
    Control,
    Plain,
}
#[derive(Clone, Copy)]
struct Lookahead {
    current: char,
    next: Option<char>,
}
impl Lookahead {
    fn substitution(self) -> bool {
        self.current == '$' && self.next == Some('(')
    }
}
fn role(quote: Option<char>, lookahead: Lookahead) -> Role {
    let ch = lookahead.current;
    let escape_start = ch == '\\' && quote != Some('\'');
    let quote_start = ch == '\'' || ch == '"';
    let shell_control = ";|&<>\n\u{60}".contains(ch) || lookahead.substitution();
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
        let lookahead = Lookahead {
            current: ch,
            next: chars.get(index + 1).copied(),
        };
        match role(quote, lookahead) {
            Role::StartEscape => escaped = true,
            Role::InQuote(q) => {
                let quote_end = ch == q;
                let opens_substitution = ch == '\u{60}' || lookahead.substitution();
                let quoted_substitution = q == '"' && opens_substitution;
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
        bail!("Agent shell commands must be one top-level just invocation.");
    }
    Ok(shell_words::split(command)?)
}
pub fn arguments(command: Option<&str>) -> Result<Vec<String>> {
    let command =
        command.ok_or_else(|| anyhow::anyhow!("one top-level just invocation is required"))?;
    let argv = words(command)?;
    let not_just = argv
        .first()
        .and_then(|arg| Path::new(arg).file_name())
        .is_none_or(|name| name != "just");
    if not_just {
        bail!("Direct shell commands are disabled; use a recipe from just --list.");
    }
    Ok(argv)
}

// DECISION: D047
pub fn enters_check(argv: &[String]) -> Result<bool> {
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

pub fn hook_commands(context: &Context, argv: Vec<String>) -> Result<Vec<String>> {
    match argv.get(1).map(String::as_str).unwrap_or("list") {
        "run" => {
            let name = argv
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("just run requires a catalog command"))?;
            command_runner::argv(context, name, forwarded(&argv[3..]))?;
        }
        "list" | "--list" | "resume" | "config-check" | "doctor" | "lint-rules" | "report"
        | "feature-merge" => {
            anyhow::ensure!(
                argv.len() <= 2,
                "one Just recipe is required; this recipe takes no arguments"
            );
        }
        "check" => {
            gate::arguments(forwarded(&argv[2..]))?;
        }
        "feature-start" => {
            let extra = forwarded(&argv[2..]);
            anyhow::ensure!(extra.len() == 1, "feature-start NAME");
        }
        "scratch" => {
            anyhow::ensure!(!forwarded(&argv[2..]).is_empty(), "scratch -- TEXT");
        }
        "lint" | "lint-config-check" | "lint-rule" | "lint-explain" | "lint-edges"
        | "lint-tree" | "lint-structure" | "jobs" | "job-status" | "job-logs" | "job-stop"
        | "job-start" | "job-cleanup" => {}
        name => {
            command_runner::argv(context, name, forwarded(&argv[2..]))?;
        }
    }
    Ok(argv)
}
