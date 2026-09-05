// DECISION: D019
pub mod git;
mod guard;
mod reminder;
mod transcript;

use crate::util::{resolve, text};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

// DECISION: D015
const SKILLS: [(&str, &str); 4] = [
    ("Plan", "edit-plan"),
    ("Decisions", "edit-decisions"),
    ("Invariants", "edit-invariants"),
    ("State", "edit-state"),
];
pub fn context(event: &str, message: &str) -> Value {
    json!({"hookSpecificOutput": {"hookEventName": event, "additionalContext": message}})
}
pub fn deny(reason: &str) -> Value {
    json!({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "deny", "permissionDecisionReason": reason}})
}
fn paths(event: &Value) -> Vec<String> {
    let payload = &event["tool_input"];
    let mut paths = Vec::new();
    for key in ["file_path", "path"] {
        if let Some(path) = payload[key].as_str() {
            paths.push(path.into());
        }
    }
    let patches = [
        payload.as_str(),
        payload["patch"].as_str(),
        payload["input"].as_str(),
        payload["patch_text"].as_str(),
    ];
    for patch in patches.into_iter().flatten() {
        for line in patch.lines() {
            for prefix in [
                "*** Add File: ",
                "*** Update File: ",
                "*** Delete File: ",
                "*** Move to: ",
            ] {
                if let Some(path) = line.strip_prefix(prefix) {
                    paths.push(path.into());
                }
            }
        }
    }
    paths
}
fn guidance(root: &Path, event: &Value, all: bool) -> String {
    let mut names = BTreeSet::new();
    let cwd = root.join(text(event, "cwd"));
    for path in paths(event) {
        if let Ok(path) = resolve(&cwd.join(path))
            && let Ok(path) = path.strip_prefix(root)
        {
            for (ledger, skill) in SKILLS {
                if path == Path::new(&format!("Ledger/{ledger}.md"))
                    || (ledger != "State"
                        && path.starts_with(format!("Ledger/{ledger}"))
                        && path.extension().is_some_and(|ext| ext == "md"))
                {
                    names.insert(skill);
                }
            }
        }
    }
    SKILLS
        .iter()
        .filter(|(_, skill)| all || names.contains(skill))
        .map(|(_, skill)| {
            format!(
                "Before editing the corresponding Ledger file, read and apply {}.",
                root.join(format!(".agents/skills/{skill}/SKILL.md"))
                    .display()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn dispatch(root: &Path, event: &Value) -> Result<Option<Value>> {
    if !event.is_object() {
        bail!("expected a hook event object");
    }
    if text(event, "hook_event_name") == "SessionStart" {
        let message = format!(
            "Resume from {} and {}. Compare the recorded task, VAC, checks, blockers, and next action with current Git status, diff, and recent commits before acting. State may be stale after an interruption; current contracts and Git take precedence. Missing State is a recovery task, not evidence that previous work completed.\n\n{}",
            root.join("Ledger/State.md").display(),
            root.join("Ledger/Plan.md").display(),
            reminder::FULL
        );
        if let Err(error) = reminder::start(event) {
            eprintln!("session reminder state: {error}");
        }
        return Ok(Some(context("SessionStart", &message)));
    }
    if text(event, "hook_event_name") != "PreToolUse" {
        return Ok(None);
    }
    let tool = text(event, "tool_name");
    if ["Bash", "Shell", "exec_command"].contains(&tool) {
        let argv = match guard::validate(root, guard::command(event)) {
            Ok(args) => args,
            Err(error) => return Ok(Some(deny(&error.to_string()))),
        };
        return Ok(if argv.get(1).is_some_and(|arg| arg == "write") {
            Some(context(
                "PreToolUse",
                &format!(
                    "Shell write targets are opaque. If this command changes Ledger, apply only the matching editing skill before the write:\n{}",
                    guidance(root, event, true)
                ),
            ))
        } else {
            None
        });
    }
    if !["apply_patch", "Edit", "Write"].contains(&tool) {
        return Ok(None);
    }
    let message = guidance(root, event, false);
    match reminder::before(root, event) {
        Ok(Some(mut reason)) => {
            if !message.is_empty() {
                reason.push_str(&format!("\n\n{message}"));
            }
            return Ok(Some(deny(&reason)));
        }
        Err(error) => eprintln!("complexity reminder: {error}"),
        _ => {}
    }
    Ok(if message.is_empty() {
        None
    } else {
        Some(context("PreToolUse", &message))
    })
}
