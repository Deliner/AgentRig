use super::{Lifetime, Record, owner, stop, storage};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn run(runtime: &Path, branch: Option<&str>) -> Result<Value> {
    let owner = owner().context("cleanup requires WORKER_OWNER or an agent session identity")?;
    let current = fs::read_to_string("/proc/self/cgroup")?;
    let mut stopped = Vec::new();
    let mut retained = Vec::new();
    let mut errors = Vec::new();
    for record in storage::list(runtime)? {
        let selected = record.owner == owner
            && branch.is_none_or(|branch| record.branch.as_deref() == Some(branch));
        let outside = !selected;
        if outside {
            continue;
        }
        let finished = record.scope.is_none() && record.finished.is_some();
        if finished {
            continue;
        }
        if let Some(reason) = retention(&record, &current) {
            retained.push(json!({"run_id": record.run_id, "reason": reason}));
            continue;
        }
        match stop::run(runtime, &record.run_id) {
            Ok(()) => stopped.push(record.run_id),
            Err(error) => errors.push(json!({"run_id": record.run_id, "error": error.to_string()})),
        }
    }
    Ok(
        json!({"owner": owner, "branch": branch, "stopped": stopped, "retained": retained, "errors": errors}),
    )
}
fn retention(record: &Record, current: &str) -> Option<&'static str> {
    match (record.lifetime, &record.scope) {
        (Lifetime::Shared, _) => Some("shared service"),
        (_, None) => Some("uncontained run; inspect manually"),
        (_, Some(scope)) if current.trim().ends_with(&format!("/{}", scope.unit)) => {
            Some("cleanup caller")
        }
        _ => None,
    }
}
