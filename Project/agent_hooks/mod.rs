// DECISION: D020
// DECISION: D015
// DECISION: D014
// DECISION: D013
// DECISION: D011
// DECISION: D003
// DECISION: D019
// DECISION: D047
pub mod input;
mod recovery;
mod reminder;
mod responses;
mod shell_guard;
mod skill_requirements;
#[cfg(test)]
mod tests;
mod tools;
mod transcript;

use self::input::{paths, target, text};
use self::responses::{Event, context};
use self::skill_requirements::Answer;
use self::tools::Tool;
use crate::change_delivery;
use crate::project_config::{Context, RouteEvent};
use crate::project_memory;
use crate::shared::calling_agent::Agent;
use crate::shared::console;
use crate::shared::vcs::Repository;
use anyhow::{Result, bail};
pub use responses::deny;
use serde_json::Value;
use std::path::{Path, PathBuf};

// DECISION: D230
const RENEWING_SOURCES: [&str; 3] = ["startup", "clear", "compact"];

// DECISION: D062
#[derive(Clone, Copy)]
enum Reminding {
    Active,
    Suppressed,
}
impl Reminding {
    fn active(self) -> bool {
        match self {
            Self::Active => true,
            Self::Suppressed => false,
        }
    }
}

// DECISION: D188
// DECISION: D200
pub fn dispatch(root: &Path, event: &Value, agent: Agent) -> Result<Option<Value>> {
    let invalid_event = !event.is_object();
    if invalid_event {
        bail!("expected a hook event object");
    }
    let mut configured = match Context::load(root) {
        Ok(configured) => configured,
        Err(error) => return Ok(recovery::respond(root, agent, event, &error)),
    };
    configured.caller.agent = Some(agent);
    match text(event, "hook_event_name") {
        "SessionStart" => session_start(root, event, &configured),
        "PreToolUse" => tool_event(root, event, &configured),
        _ => Ok(None),
    }
}
fn tool_event(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let tool = tools::classify(event);
    let awaited = tool
        .waits_for_compaction()
        .then(|| compaction_refusal(configured))
        .flatten();
    if let Some(refusal) = awaited {
        return Ok(Some(refusal));
    }
    match tool {
        Tool::Shell => shell_event(event, configured),
        Tool::Edit => edit_event(root, event, configured),
        Tool::Passive | Tool::Compact => Ok(None),
        Tool::Refused => Ok(Some(deny(&tools::refusal(event)))),
    }
}
// DECISION: D230
fn compaction_refusal(configured: &Context) -> Option<Value> {
    match change_delivery::compaction_due(configured) {
        Ok(due) => due.map(deny),
        Err(error) => Some(deny(&format!(
            "AgentRig refused this call: it could not read whether a merge requires compacting this conversation: {error:#}. Ask the user to repair the runtime directory {}.",
            configured.config.paths.runtime
        ))),
    }
}
// DECISION: D230
fn lift_compaction(event: &Value, configured: &Context) -> Option<String> {
    let renewed = RENEWING_SOURCES.contains(&text(event, "source"));
    let error = renewed
        .then(|| change_delivery::lift_compaction(configured).err())
        .flatten()?;
    console::stderr(format!("compaction requirement: {error:#}"));
    Some(format!(
        "AgentRig could not lift the compaction requirement of the last merge: {error:#}. Every tool call stays refused; ask the user to repair the runtime directory {}.",
        configured.config.paths.runtime
    ))
}
fn session_start(root: &Path, event: &Value, configured: &Context) -> Result<Option<Value>> {
    let memory = &configured.config.paths.memory;
    let full = full_refresh(configured);
    let card = match active_card(root, configured) {
        Ok(card) => card.display().to_string(),
        Err(error) => format!("the active card, which AgentRig could not find: {error:#}"),
    };
    let mut message = format!(
        "Resume from {state} and {card}. Compare the recorded task, VAC, checks, blockers, and next action with current Git status, diff, and recent commits before acting. State may be stale after an interruption; current contracts and Git take precedence. Missing State is a recovery task, not evidence that previous work completed.\n\n{full}",
        state = root.join(memory).join("State.md").display(),
    );
    if let Some(failure) = lift_compaction(event, configured) {
        message.push_str(&format!("\n\n{failure}"));
    }
    let hooks = &configured.config.hooks;
    let stateful = hooks.reminder.is_some() || !hooks.routes.is_empty();
    let started = stateful
        .then(|| reminder::start_at(event, &state_directory(configured)?))
        .transpose();
    if let Err(error) = started {
        console::stderr(format!("session reminder state: {error}"));
        message.push_str(&format!("\n\n{}", state_failure(configured, &error)));
    }
    Ok(Some(context(Event::SessionStart, &message)))
}
fn active_card(root: &Path, configured: &Context) -> Result<PathBuf> {
    let repository = Repository::discover(root)?;
    let observed = repository.as_ref().map(Repository::observe).transpose()?;
    let branch = observed.map(|observed| observed.branch).unwrap_or_default();
    project_memory::active_card(configured, &branch)
}
fn shell_event(event: &Value, configured: &Context) -> Result<Option<Value>> {
    // DECISION: D201
    let inspected = shell_guard::arguments(shell_guard::command(event))
        .and_then(|argv| shell_guard::inspect(configured, &argv));
    let shell = match inspected {
        Ok(shell) => shell,
        Err(error) => return Ok(Some(deny(&error.to_string()))),
    };
    if let Some(refusal) = protected(configured, &shell.named) {
        return Ok(Some(refusal));
    }
    let opaque = shell.catalog.as_deref().is_some_and(|name| {
        configured
            .config
            .commands
            .get(name)
            .is_some_and(|command| !command.access.read_only())
    });
    let mut found = Vec::new();
    if opaque {
        // DECISION: D196
        found.extend(match shell.written {
            Some(paths) => skill_requirements::writes(configured, &paths),
            None => skill_requirements::events(configured, RouteEvent::Shell, "opaque shell write"),
        });
    }
    if shell.check {
        found.extend(skill_requirements::events(
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
    if let Some(refusal) = protected(configured, &paths) {
        return Ok(Some(refusal));
    }
    let found = skill_requirements::edits(configured, &paths)?;
    respond(event, configured, found, Reminding::Active)
}
// DECISION: D213
fn protected(configured: &Context, paths: &[PathBuf]) -> Option<Value> {
    let mut touched: Vec<String> = paths
        .iter()
        .filter(|path| configured.protection.closed_to_agents(path))
        .map(|path| shown(path))
        .collect();
    touched.dedup();
    let allowed = touched.is_empty();
    if allowed {
        return None;
    }
    let skill = configured.protection_skill();
    Some(deny(&format!(
        "AgentRig refused this call: it changes the protected enforcement files {}. Agents do not change them directly; read and apply {skill} to propose the change for the user's approval.",
        touched.join(", ")
    )))
}
fn shown(path: &Path) -> String {
    let root = path.as_os_str().is_empty();
    if root {
        ".".into()
    } else {
        path.display().to_string()
    }
}
// DECISION: D047
fn respond(
    event: &Value,
    configured: &Context,
    found: Vec<skill_requirements::Occurrence>,
    reminding: Reminding,
) -> Result<Option<Value>> {
    let answer = match reminder::open(event, &state_directory(configured)?) {
        Ok(mut session) => {
            let mut answer = remind(event, configured, session.state_mut(), reminding);
            answer.found = skill_requirements::pending(session.state_mut(), configured, found);
            if let Err(error) = session.save() {
                console::stderr(format!("complexity reminder: {error}"));
                answer.discipline = None;
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
            }
        }
    };
    Ok(answer.output(configured, text(event, "tool_name")))
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
        reminder::Schedule::read(&config)?.due(event, state)
    });
    let mut answer = Answer {
        discipline: None,
        found: Vec::new(),
        notes: Vec::new(),
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
        "Hook session state unavailable: {error}. Ask the user to repair the state storage at {} or the hook transcript_path. Until then a call that needs a required skill stops every time, because its delivery cannot be recorded; calls without one proceed.",
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
        .map(|skill| reminder::FULL.replace("{discipline_skill}", &context.caller.skill(skill)))
        .unwrap_or_default()
}
