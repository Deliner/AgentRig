// DECISION: D045
// DECISION: D046
use super::input::{paths, target, text};
use super::responses::{Event, context, deny};
use super::shell_guard;
use super::tools::{self, Tool};
use crate::project_config;
use crate::shared::{calling_agent::Agent, diagnostics};
use serde_json::Value;
use std::path::{Path, PathBuf};

// DECISION: D188
// DECISION: D200
pub(super) fn respond(
    root: &Path,
    agent: Agent,
    event: &Value,
    error: &anyhow::Error,
) -> Option<Value> {
    let sources = project_config::sources(root);
    let editable = editable(&sources);
    let skill = diagnostics::config_skill(root, Some(agent));
    let shown = skill
        .clone()
        .unwrap_or_else(|| diagnostics::UNNAMED_SKILL.to_owned());
    let message = message(error, &editable, &shown);
    match (text(event, "hook_event_name"), tools::classify(event)) {
        ("SessionStart", _) => Some(context(Event::SessionStart, &message)),
        ("PreToolUse", Tool::Shell) => {
            let command = shell_guard::command(event);
            let argv = shell_guard::arguments(command).unwrap_or_default();
            let check = matches!(argv.as_slice(), [_, recipe] if recipe == "config-check");
            let words = command.and_then(|command| shell_guard::words(command).ok());
            let named = skill.and_then(|skill| target(root, event, &skill));
            let readable = [sources, named.into_iter().collect()].concat();
            let read = match words.as_deref() {
                Some([cat, files @ ..]) => cat == "cat" && within(root, event, files, &readable),
                _ => false,
            };
            (!check && !read).then(|| deny(&message))
        }
        ("PreToolUse", Tool::Edit) => {
            let configuration = within(root, event, &paths(event), &editable);
            (!configuration).then(|| deny(&message))
        }
        ("PreToolUse", Tool::Refused) => Some(deny(&tools::refusal(event))),
        _ => None,
    }
}

fn within(root: &Path, event: &Value, files: &[String], allowed: &[PathBuf]) -> bool {
    !files.is_empty()
        && files
            .iter()
            .all(|file| target(root, event, file).is_some_and(|file| allowed.contains(&file)))
}

// DECISION: D213
fn editable(sources: &[PathBuf]) -> Vec<PathBuf> {
    let protected = project_config::protected_configuration();
    sources
        .iter()
        .filter(|source| !protected.iter().any(|path| *source == Path::new(path)))
        .cloned()
        .collect()
}

fn message(error: &anyhow::Error, editable: &[PathBuf], skill: &str) -> String {
    let editable: Vec<_> = editable
        .iter()
        .map(|path| path.display().to_string())
        .collect();
    let protected = project_config::protected_configuration().join(", ");
    format!(
        "AgentRig configuration error: {error:#}. Until `just config-check` passes, only edits of {editable}, `cat` of these files, {protected} and {skill}, and `just config-check` are available. ACTION: Apply {skill}: correct the reported file directly, then run `just config-check`. Agents do not change the protected {protected}; when the error lies there, ask the user to correct it.",
        editable = editable.join(", ")
    )
}
