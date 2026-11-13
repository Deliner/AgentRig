// DECISION: D045
// DECISION: D046
use super::{Event, context, deny, input::text, paths, shell_guard, target};
use crate::{project_config, shared::diagnostics};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) fn respond(root: &Path, event: &Value, error: &anyhow::Error) -> Option<Value> {
    let sources = project_config::sources(root);
    let skill = diagnostics::config_skill(root);
    let message = message(error, &sources, &skill);
    match (text(event, "hook_event_name"), text(event, "tool_name")) {
        ("SessionStart", _) => Some(context(Event::SessionStart, &message)),
        ("PreToolUse", "Bash" | "Shell" | "exec_command") => {
            let command = shell_guard::command(event);
            let argv = shell_guard::arguments(command).unwrap_or_default();
            let check = matches!(argv.as_slice(), [_, recipe] if recipe == "config-check");
            let words = command.and_then(|command| shell_guard::words(command).ok());
            let readable = [sources, vec![PathBuf::from(skill)]].concat();
            let read = match words.as_deref() {
                Some([cat, files @ ..]) => cat == "cat" && within(root, event, files, &readable),
                _ => false,
            };
            (!check && !read).then(|| deny(&message))
        }
        ("PreToolUse", "apply_patch" | "Edit" | "Write") => {
            let configuration = within(root, event, &paths(event), &sources);
            (!configuration).then(|| deny(&message))
        }
        _ => None,
    }
}

fn within(root: &Path, event: &Value, files: &[String], allowed: &[PathBuf]) -> bool {
    !files.is_empty()
        && files
            .iter()
            .all(|file| target(root, event, file).is_some_and(|file| allowed.contains(&file)))
}

fn message(error: &anyhow::Error, sources: &[PathBuf], skill: &str) -> String {
    let editable: Vec<_> = sources
        .iter()
        .map(|source| source.display().to_string())
        .collect();
    format!(
        "AgentRig configuration error: {error:#}. Until `just config-check` passes, only edits of {}, `cat` of these files and {skill}, and `just config-check` are available. ACTION: Apply {skill}: correct the reported file directly, then run `just config-check`.",
        editable.join(", ")
    )
}
