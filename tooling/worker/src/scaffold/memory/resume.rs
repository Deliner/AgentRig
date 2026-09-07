// DECISION: D013
// DECISION: D022
use super::super::{config::Context, evidence};
use anyhow::Result;
use review_runner::vcs::Repository;
use serde_json::{Value, json};
use std::fs;

pub fn run(context: &Context) -> Result<()> {
    let memory = context.path(&context.config.paths.memory)?;
    let state = fs::read_to_string(memory.join("State.md"))?;
    let plan = fs::read_to_string(memory.join("Plan.md"))?;
    let repository = Repository::discover(&context.root)?;
    let observed = repository.as_ref().map(Repository::observe).transpose()?;
    let branch = observed.as_ref().map_or("", |value| value.branch.as_str());
    let revision = observed
        .as_ref()
        .map_or("", |value| value.revision.as_str());
    let recorded = recorded_revision(repository.as_ref(), &state, revision);
    let snapshot = snapshot(&state, branch, &recorded);
    let mut report = json!({"state": state, "plan": plan, "vcs": observed,
        "snapshot": snapshot, "state_revision": recorded, "upgrade": crate::scaffold::upgrade::recovery::journal(&context.root)?, "checks": evidence::resume(context),
        "jobs": agentrig::jobs::list(&context.path(&context.config.paths.runtime)?)?});
    let legacy_git = report["vcs"]["backend"] == "git";
    if legacy_git {
        report["git"] = report["vcs"].clone();
    }
    println!("{report}");
    Ok(())
}
fn claim<'a>(state: &'a str, prefix: &str) -> Option<&'a str> {
    state
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(|value| value.trim().trim_matches('`'))
}
fn recorded_revision(repository: Option<&Repository<'_>>, state: &str, current: &str) -> Value {
    let saved = claim(state, "Revision: ");
    let resolved = saved
        .filter(|value| {
            value.len() >= 4
                && value.len() <= 64
                && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        .and_then(|saved| repository?.resolve(saved).ok());
    let changed = resolved
        .as_deref()
        .filter(|_| !current.is_empty())
        .map(|revision| revision != current);
    json!({"recorded": saved, "resolved": resolved, "head_changed": changed})
}
fn snapshot(state: &str, branch: &str, revision: &Value) -> &'static str {
    let saved_branch = claim(state, "Branch: ");
    let saved_revision = claim(state, "Revision: ");
    let absent = saved_branch.is_none() && saved_revision.is_none();
    let changed =
        saved_branch.is_some_and(|saved| saved != branch) || revision["head_changed"] == true;
    let unresolved = saved_revision.is_some() && revision["head_changed"].is_null();
    if absent {
        "unverified"
    } else if changed {
        "stale"
    } else if unresolved {
        "unverified"
    } else {
        "current"
    }
}
