// DECISION: D020
// DECISION: D015
// DECISION: D014
// DECISION: D013
// DECISION: D011
// DECISION: D003
// DECISION: D019
mod guard;
mod reminder;
mod transcript;

use crate::scaffold::config::Context;
use crate::util::text;
use agentrig::paths::resolve;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

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
pub fn dispatch(root: &Path, event: &Value) -> Result<Option<Value>> {
    let invalid_event = !event.is_object();
    if invalid_event {
        bail!("expected a hook event object");
    }
    let recovering = text(event, "hook_event_name") == "SessionStart"
        && crate::scaffold::upgrade::recovery::configuration_pending(root)?;
    if recovering {
        return Ok(Some(context(
            "SessionStart",
            &crate::scaffold::upgrade::recovery::guidance(root)?,
        )));
    }
    let configured = Context::load_for(root, true)?;
    match text(event, "hook_event_name") {
        "SessionStart" => session_start(root, event, &configured),
        "PreToolUse" => match text(event, "tool_name") {
            "Bash" | "Shell" | "exec_command" => shell_event(root, event, &configured),
            "apply_patch" | "Edit" | "Write" => edit_event(root, event, &configured),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
fn session_start(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let memory = &configured.config.paths.memory;
    let full = format!(
        "{}\n{}",
        full_refresh(configured),
        crate::scaffold::upgrade::recovery::guidance(root)?
    );
    let message = format!(
        "Resume from {} and {}. Compare the recorded task, VAC, checks, blockers, and next action with current Git status, diff, and recent commits before acting. State may be stale after an interruption; current contracts and Git take precedence. Missing State is a recovery task, not evidence that previous work completed.\n\n{}",
        root.join(memory).join("State.md").display(),
        root.join(memory).join("Plan.md").display(),
        full
    );
    let directory = configured
        .path(&configured.config.paths.runtime)?
        .join("reminders");
    let started = reminder::start_at(event, &directory);
    if let Err(error) = started {
        eprintln!("session reminder state: {error}");
    }
    Ok(Some(context("SessionStart", &message)))
}
fn shell_event(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let validated = guard::arguments(guard::command(event))
        .and_then(|args| crate::scaffold::hook_commands(configured, args));
    let argv = match validated {
        Ok(args) => args,
        Err(error) => return Ok(Some(deny(&error.to_string()))),
    };
    let opaque = argv.get(1).is_some_and(|arg| {
        arg == "run"
            || configured
                .config
                .commands
                .get(arg)
                .is_some_and(|command| !command.read_only)
    });
    let guidance = edit_guidance(root, event, true, configured)?;
    Ok(if opaque {
        Some(context(
            "PreToolUse",
            &format!(
                "Shell write targets are opaque. If this command changes project memory, apply only the matching editing skill before the write:\n{}",
                guidance
            ),
        ))
    } else {
        None
    })
}
fn edit_event(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let message = edit_guidance(root, event, false, configured)?;
    let reminder = match &configured.config.hooks.reminder {
        Some(path) => reminder::before_at(
            event,
            &crate::util::object(&configured.path(path)?),
            &configured
                .path(&configured.config.paths.runtime)?
                .join("reminders"),
        ),
        None => Ok(None),
    };
    match reminder {
        Ok(Some(mut reason)) => {
            reason = reason.replace(reminder::FULL, &full_refresh(configured));
            let has_guidance = !message.is_empty();
            if has_guidance {
                reason.push_str(&format!("\n\n{message}"));
            }
            return Ok(Some(deny(&reason)));
        }
        Err(error) => eprintln!("complexity reminder: {error}"),
        _ => {}
    }
    let no_guidance = message.is_empty();
    Ok(if no_guidance {
        None
    } else {
        Some(context("PreToolUse", &message))
    })
}

fn full_refresh(context: &crate::scaffold::config::Context) -> String {
    context
        .config
        .hooks
        .discipline_skill
        .as_ref()
        .map(|skill| {
            reminder::FULL.replace(
                "{discipline_skill}",
                &context.root.join(skill).to_string_lossy(),
            )
        })
        .unwrap_or_default()
}
fn edit_guidance(
    root: &Path,
    event: &Value,
    all: bool,
    context: &crate::scaffold::config::Context,
) -> Result<String> {
    let cwd = root.join(text(event, "cwd"));
    let paths: Vec<_> = paths(event)
        .iter()
        .filter_map(|path| resolve(&cwd.join(path)).ok())
        .filter_map(|path| path.strip_prefix(root).ok().map(Path::to_owned))
        .collect();
    let mut skills = BTreeSet::new();
    for route in &context.config.hooks.routes {
        let include = crate::lint::config::globs(&route.include)?;
        let applies = all || paths.iter().any(|path| include.is_match(path));
        if applies {
            skills.insert(context.path(&route.skill)?);
        }
    }
    Ok(skills
        .iter()
        .map(|skill| {
            format!(
                "Before editing the corresponding memory file, read and apply {}.",
                skill.display()
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}
