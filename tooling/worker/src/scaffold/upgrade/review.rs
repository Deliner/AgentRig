use super::{
    model::{Action, Change, Plan, Resolution, State},
    release, storage,
};
use crate::scaffold::package::manifest;
use anyhow::{Context as _, Result, ensure};
use std::{collections::BTreeSet, fs, path::Path};

pub fn load(root: &Path, path: &Path) -> Result<Plan> {
    let mut plan: Plan = serde_json::from_slice(&fs::read(path)?)?;
    ensure!(
        plan.version == 1 && plan.from_version == release::FROM && plan.to_version == release::TO,
        "unsupported upgrade plan"
    );
    ensure!(
        Path::new(&plan.project) == root,
        "plan belongs to another project"
    );
    ensure!(
        plan.checks == ["config-check", "doctor", "check"],
        "plan must retain all post-upgrade checks"
    );
    let directory = path.parent().context("plan directory required")?;
    let mut resolved = BTreeSet::new();
    for (name, change) in &mut plan.files {
        verify_state(root, name, change, &mut resolved)?;
        let kept = resolve(name, change)?;
        if kept {
            plan.manifest
                .local
                .insert(name.clone(), change.before.sha256.clone());
        }
        verify_payload(directory, &change.before)?;
        verify_payload(directory, &change.after)?;
    }
    receipt(&mut plan, directory)?;
    Ok(plan)
}
fn verify_state(
    root: &Path,
    name: &str,
    change: &Change,
    resolved: &mut BTreeSet<String>,
) -> Result<()> {
    let current = storage::state(root, name)?;
    ensure!(
        current == change.before,
        "file changed since planning: {name}"
    );
    ensure!(
        change.after.resolved == current.resolved,
        "plan redirects a file: {name}"
    );
    ensure!(
        resolved.insert(current.resolved.clone()),
        "overlapping upgrade target: {name}"
    );
    ensure!(
        !root
            .join(&current.resolved)
            .starts_with(storage::directory(root)?),
        "upgrade cannot overwrite its recovery data"
    );
    Ok(())
}
fn resolve(name: &str, change: &mut Change) -> Result<bool> {
    let conflict = change.action == Action::Conflict;
    if conflict {
        let resolution = change.resolution.as_ref().with_context(|| {
            format!("unresolved conflict: {name}; set resolution to keep or replace")
        })?;
        let keep = *resolution == Resolution::Keep;
        if keep {
            change.after = change.before.clone();
        }
        Ok(keep)
    } else {
        ensure!(
            change.resolution.is_none(),
            "resolution only applies to conflicts: {name}"
        );
        Ok(false)
    }
}
fn verify_payload(directory: &Path, state: &State) -> Result<()> {
    if let Some(hash) = &state.sha256 {
        storage::payload(directory, hash)?;
        ensure!(state.mode.is_some(), "file mode required");
    }
    Ok(())
}
fn receipt(plan: &mut Plan, directory: &Path) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(&plan.manifest)?;
    let change = plan
        .files
        .get_mut(manifest::PATH)
        .context("plan must include installation receipt")?;
    change.after.sha256 = Some(storage::blob(directory, &bytes)?);
    change.after.mode = Some(0o644);
    change.action = Action::Replace;
    Ok(())
}
