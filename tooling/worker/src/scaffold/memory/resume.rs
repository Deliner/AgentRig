// DECISION: D013
// DECISION: D022
use super::super::{config::Context, evidence};
use anyhow::Result;
use serde_json::{Value, json};
use std::fs;

pub fn run(context: &Context) -> Result<()> {
    let memory = context.path(&context.config.paths.memory)?;
    let state = fs::read_to_string(memory.join("State.md"))?;
    let plan = fs::read_to_string(memory.join("Plan.md"))?;
    let git = |args: &[&str]| crate::util::git(&context.root, args).unwrap_or_default();
    let branch = git(&["branch", "--show-current"]);
    let revision = git(&["rev-parse", "HEAD"]);
    let status = git(&["--no-optional-locks", "status", "--short"]);
    let recorded = recorded_revision(context, &state, &revision);
    let snapshot = snapshot(&state, &branch, &recorded);
    println!(
        "{}",
        json!({"state": state, "plan": plan,
        "git": {"branch": branch, "revision": revision, "status": status,
            "merge_in_progress": operation(context, "MERGE_HEAD"),
            "rebase_in_progress": operation(context, "rebase-merge") || operation(context, "rebase-apply")},
        "snapshot": snapshot, "state_revision": recorded, "upgrade": crate::scaffold::upgrade::recovery::journal(&context.root)?, "checks": evidence::resume(context),
        "jobs": discipline_worker::jobs::list(&context.path(&context.config.paths.runtime)?)?})
    );
    Ok(())
}
fn claim<'a>(state: &'a str, prefix: &str) -> Option<&'a str> {
    state
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(|value| value.trim().trim_matches('`'))
}
fn recorded_revision(context: &Context, state: &str, current: &str) -> Value {
    let saved = claim(state, "Revision: ");
    let resolved = saved
        .filter(|value| {
            value.len() >= 4
                && value.len() <= 64
                && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        .and_then(|saved| {
            crate::util::git(
                &context.root,
                &[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{saved}^{{commit}}"),
                ],
            )
            .ok()
        });
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
fn operation(context: &Context, name: &str) -> bool {
    let path = crate::util::git(&context.root, &["rev-parse", "--git-path", name]);
    match path {
        Ok(path) => context.root.join(path).exists(),
        Err(_) => false,
    }
}
