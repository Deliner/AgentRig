// DECISION: D045
use super::{context, deny, guard, input::text, paths, target};
use crate::{project_config, shared::diagnostics};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) fn respond(root: &Path, event: &Value, error: &anyhow::Error) -> Option<Value> {
    let sources = project_config::sources(root);
    let message = message(root, error, &sources);
    match (text(event, "hook_event_name"), text(event, "tool_name")) {
        ("SessionStart", _) => Some(context("SessionStart", &message)),
        ("PreToolUse", "Bash" | "Shell" | "exec_command") => {
            let argv = guard::arguments(guard::command(event)).unwrap_or_default();
            let check = matches!(argv.as_slice(), [_, recipe] if recipe == "config-check");
            (!check).then(|| deny(&message))
        }
        ("PreToolUse", "apply_patch" | "Edit" | "Write") => {
            let edited = paths(event);
            let configuration = !edited.is_empty()
                && edited.iter().all(|path| {
                    target(root, event, path).is_some_and(|path| sources.contains(&path))
                });
            (!configuration).then(|| deny(&message))
        }
        _ => None,
    }
}

fn message(root: &Path, error: &anyhow::Error, sources: &[PathBuf]) -> String {
    let editable: Vec<_> = sources
        .iter()
        .map(|source| source.display().to_string())
        .collect();
    format!(
        "AgentRig configuration error: {error:#}. Until `just config-check` passes, only edits of {} and `just config-check` are available. ACTION: Apply {}: correct the reported file directly, then run `just config-check`.",
        editable.join(", "),
        diagnostics::config_skill(root)
    )
}
