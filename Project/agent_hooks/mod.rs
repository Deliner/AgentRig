// DECISION: D020
// DECISION: D015
// DECISION: D014
// DECISION: D013
// DECISION: D011
// DECISION: D003
// DECISION: D019
// DECISION: D046
// DECISION: D047
mod guard;
pub mod input;
mod recovery;
mod reminder;
mod requirements;
#[cfg(test)]
mod tests;
mod transcript;

use self::input::text;
use self::requirements::Answer;
use crate::project_config::{Access, Context, RouteEvent};
use crate::shared::console;
use crate::shared::paths::resolve;
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

// DECISION: D062
#[derive(Clone, Copy)]
enum Reminding {
    Active,
    Suppressed,
}
impl Reminding {
    fn active(self) -> bool {
        matches!(self, Self::Active)
    }
}

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
        payload["command"].as_str(),
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
fn target(root: &Path, event: &Value, path: &str) -> Option<PathBuf> {
    let cwd = root.join(text(event, "cwd"));
    let resolved = resolve(&cwd.join(path)).ok()?;
    resolved.strip_prefix(root).ok().map(Path::to_owned)
}
pub fn dispatch(root: &Path, event: &Value) -> Result<Option<Value>> {
    let invalid_event = !event.is_object();
    if invalid_event {
        bail!("expected a hook event object");
    }
    let configured = match Context::load(root) {
        Ok(configured) => configured,
        Err(error) => return Ok(recovery::respond(root, event, &error)),
    };
    match text(event, "hook_event_name") {
        "SessionStart" => session_start(root, event, &configured),
        "PreToolUse" => match text(event, "tool_name") {
            "Bash" | "Shell" | "exec_command" => shell_event(event, &configured),
            "apply_patch" | "Edit" | "Write" => edit_event(root, event, &configured),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
fn session_start(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let memory = &configured.config.paths.memory;
    let full = full_refresh(configured);
    let mut message = format!(
        "Resume from {} and {}. Compare the recorded task, VAC, checks, blockers, and next action with current Git status, diff, and recent commits before acting. State may be stale after an interruption; current contracts and Git take precedence. Missing State is a recovery task, not evidence that previous work completed.\n\n{}",
        root.join(memory).join("State.md").display(),
        root.join(memory).join("Plan.md").display(),
        full
    );
    let hooks = &configured.config.hooks;
    let stateful = hooks.reminder.is_some() || !hooks.routes.is_empty();
    let started = stateful
        .then(|| reminder::start_at(event, &state_directory(configured)?))
        .transpose();
    if let Err(error) = started {
        console::stderr(format!("session reminder state: {error}"));
        message.push_str(&format!("\n\n{}", state_failure(configured, &error)));
    }
    Ok(Some(context("SessionStart", &message)))
}
fn shell_event(event: &Value, configured: &Context) -> Result<Option<Value>> {
    let validated = guard::arguments(guard::command(event))
        .and_then(|args| guard::hook_commands(configured, args))
        .and_then(|args| guard::enters_check(&args).map(|check| (args, check)));
    let (argv, check) = match validated {
        Ok(validated) => validated,
        Err(error) => return Ok(Some(deny(&error.to_string()))),
    };
    let catalog = match argv.get(1).map(String::as_str) {
        Some("run") => argv.get(2),
        _ => argv.get(1),
    };
    let opaque = catalog.is_some_and(|name| {
        name == "scratch"
            || configured
                .config
                .commands
                .get(name)
                .is_some_and(|command| command.access == Access::Writable)
    });
    let mut found = Vec::new();
    if opaque {
        found.extend(requirements::events(
            configured,
            RouteEvent::Shell,
            "opaque shell write",
        ));
    }
    if check {
        found.extend(requirements::events(
            configured,
            RouteEvent::Check,
            "VAC check entry",
        ));
    }
    respond(event, configured, found, Reminding::Suppressed)
}
fn edit_event(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let paths: Vec<_> = paths(event)
        .iter()
        .filter_map(|path| target(root, event, path))
        .collect();
    let found = requirements::edits(configured, &paths)?;
    respond(event, configured, found, Reminding::Active)
}
// DECISION: D047
fn respond(
    event: &Value,
    configured: &Context,
    found: Vec<requirements::Occurrence>,
    reminding: Reminding,
) -> Result<Option<Value>> {
    let answer = match reminder::open(event, &state_directory(configured)?) {
        Ok(mut session) => {
            let mut answer = remind(event, configured, session.state_mut(), reminding);
            answer.found = requirements::pending(session.state_mut(), configured, found);
            if let Err(error) = session.save() {
                console::stderr(format!("complexity reminder: {error}"));
                answer.discipline = None;
                answer.tracked = false;
                answer.notes.push(state_failure(configured, &error));
            }
            answer
        }
        Err(error) => {
            console::stderr(format!("complexity reminder: {error}"));
            Answer {
                discipline: None,
                found,
                notes: vec![state_failure(configured, &error)],
                tracked: false,
            }
        }
    };
    answer.output(configured, text(event, "tool_name"))
}
fn remind(event: &Value, configured: &Context, state: &mut Value, reminding: Reminding) -> Answer {
    let schedule = configured
        .config
        .hooks
        .reminder
        .as_ref()
        .filter(|_| reminding.active());
    let due = schedule.map_or(Ok(None), |path| {
        let config = input::object(&configured.path(path)?);
        reminder::due(event, &config, state)
    });
    let mut answer = Answer {
        discipline: None,
        found: Vec::new(),
        notes: Vec::new(),
        tracked: true,
    };
    match due {
        Ok(due) => {
            answer.discipline =
                due.map(|text| text.replace(reminder::FULL, &full_refresh(configured)));
        }
        Err(error) => {
            console::stderr(format!("complexity reminder: {error}"));
            answer.notes.push(state_failure(configured, &error));
        }
    }
    answer
}
fn state_directory(configured: &Context) -> Result<PathBuf> {
    Ok(configured
        .path(&configured.config.paths.runtime)?
        .join("reminders"))
}
fn state_failure(configured: &Context, error: &anyhow::Error) -> String {
    format!(
        "Hook session state unavailable: {error}. Check the state storage at {} and the hook transcript_path; repair the unavailable input and retry. Until then requirements repeat without stopping actions; ordinary file editing remains available.",
        configured
            .root
            .join(&configured.config.paths.runtime)
            .join("reminders")
            .display()
    )
}

fn full_refresh(context: &crate::project_config::Context) -> String {
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
