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
    let configured = if root.join(crate::scaffold::config::FILE).is_file() {
        Some(crate::scaffold::config::Context::load(root)?)
    } else {
        None
    };
    if text(event, "hook_event_name") == "SessionStart" {
        let memory = configured
            .as_ref()
            .map(|ctx| ctx.config.paths.memory.as_str())
            .unwrap_or("Ledger");
        let full = portable_full(configured.as_ref());
        let message = format!(
            "Resume from {} and {}. Compare the recorded task, VAC, checks, blockers, and next action with current Git status, diff, and recent commits before acting. State may be stale after an interruption; current contracts and Git take precedence. Missing State is a recovery task, not evidence that previous work completed.\n\n{}",
            root.join(memory).join("State.md").display(),
            root.join(memory).join("Plan.md").display(),
            full
        );
        let directory = configured
            .as_ref()
            .map(|ctx| {
                ctx.path(&ctx.config.paths.runtime)
                    .map(|p| p.join("reminders"))
            })
            .transpose()?;
        let started = if configured.is_some() {
            reminder::start_at(event, directory.as_deref())
        } else {
            reminder::start(event)
        };
        if let Err(error) = started {
            eprintln!("session reminder state: {error}");
        }
        return Ok(Some(context("SessionStart", &message)));
    }
    if text(event, "hook_event_name") != "PreToolUse" {
        return Ok(None);
    }
    let tool = text(event, "tool_name");
    if ["Bash", "Shell", "exec_command"].contains(&tool) {
        let validated = match &configured {
            Some(context) => guard::arguments(guard::command(event))
                .and_then(|args| crate::scaffold::hook_commands(context, args)),
            None => guard::validate(root, guard::command(event)),
        };
        let argv = match validated {
            Ok(args) => args,
            Err(error) => return Ok(Some(deny(&error.to_string()))),
        };
        let opaque = if configured.is_some() {
            argv.get(1).is_some_and(|arg| arg == "run")
        } else {
            argv.get(1).is_some_and(|arg| arg == "write")
        };
        let edit_guidance = portable_guidance(root, event, true, configured.as_ref())?;
        return Ok(if opaque {
            Some(context(
                "PreToolUse",
                &format!(
                    "Shell write targets are opaque. If this command changes project memory, apply only the matching editing skill before the write:\n{}",
                    edit_guidance
                ),
            ))
        } else {
            None
        });
    }
    if !["apply_patch", "Edit", "Write"].contains(&tool) {
        return Ok(None);
    }
    let message = portable_guidance(root, event, false, configured.as_ref())?;
    let reminder = if let Some(context) = &configured {
        match &context.config.hooks.reminder {
            Some(path) => reminder::before_at(
                event,
                &crate::util::object(&context.path(path)?),
                Some(
                    &context
                        .path(&context.config.paths.runtime)?
                        .join("reminders"),
                ),
            ),
            None => Ok(None),
        }
    } else {
        reminder::before(root, event)
    };
    match reminder {
        Ok(Some(mut reason)) => {
            reason = reason.replace(reminder::FULL, &portable_full(configured.as_ref()));
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

fn portable_full(context: Option<&crate::scaffold::config::Context>) -> String {
    match context.and_then(|ctx| {
        ctx.config
            .hooks
            .discipline_skill
            .as_ref()
            .map(|skill| ctx.root.join(skill))
    }) {
        Some(path) => reminder::FULL.replace(
            ".agents/skills/complexity-discipline/SKILL.md",
            &path.to_string_lossy(),
        ),
        None if context.is_some() => String::new(),
        None => reminder::FULL.into(),
    }
}
fn portable_guidance(
    root: &Path,
    event: &Value,
    all: bool,
    context: Option<&crate::scaffold::config::Context>,
) -> Result<String> {
    let Some(context) = context else {
        return Ok(guidance(root, event, all));
    };
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
