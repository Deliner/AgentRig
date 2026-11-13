// DECISION: D013
// DECISION: D022
use super::check_evidence;
use crate::project_config::Context;
use crate::shared::console;
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::Repository;
use anyhow::Result;
use serde_json::{Value, json};

pub fn run(context: &Context) -> Result<()> {
    let memory = context.path(&context.config.paths.memory)?;
    let state = fs::read_to_string(memory.join("State.md"))?;
    let plan = fs::read_to_string(memory.join("Plan.md"))?;
    let repository = crate::shared::vcs::Repository::discover(&context.root)?;
    let observed = repository.as_ref().map(Repository::observe).transpose()?;
    let branch = observed.as_ref().map_or("", |value| value.branch.as_str());
    let revision = observed
        .as_ref()
        .map_or("", |value| value.revision.as_str());
    let claims = Recorded(&state);
    let recorded = claims.revision(repository.as_ref(), revision);
    let snapshot = claims.snapshot(branch, &recorded);
    let report = json!({"state": state, "plan": plan, "vcs": observed,
        "snapshot": snapshot, "state_revision": recorded, "checks": check_evidence::resume(context),
        "jobs": crate::command_runner::jobs::list(&context.path(&context.config.paths.runtime)?)?});
    console::stdout(report);
    Ok(())
}
#[derive(Clone, Copy)]
struct Recorded<'a>(&'a str);
impl<'a> Recorded<'a> {
    fn claim(self, prefix: &str) -> Option<&'a str> {
        self.0
            .lines()
            .find_map(|line| line.strip_prefix(prefix))
            .map(|value| value.trim().trim_matches('`'))
    }
    fn revision(self, repository: Option<&Repository<'_>>, current: &str) -> Value {
        let saved = self.claim("Revision: ");
        let resolved = saved.and_then(|saved| repository?.resolve(saved).ok());
        let changed = resolved
            .as_deref()
            .filter(|_| !current.is_empty())
            .map(|revision| revision != current);
        json!({"recorded": saved, "resolved": resolved, "head_changed": changed})
    }
    fn snapshot(self, branch: &str, revision: &Value) -> &'static str {
        let saved_branch = self.claim("Branch: ");
        let changed =
            saved_branch.is_some_and(|saved| saved != branch) || revision["head_changed"] == true;
        let unresolved = revision["head_changed"].is_null();
        if changed {
            "stale"
        } else if unresolved {
            "unverified"
        } else {
            "current"
        }
    }
}
