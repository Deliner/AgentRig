use super::{operation::Operation, storage};
use crate::scaffold::{commands::process, config};
use anyhow::{Result, ensure};
use std::path::Path;

pub fn run(root: &Path, path: &Path) -> Result<i32> {
    let mut operation = Operation::prepare(root, path)?;
    let applied = operation.journal.phase == "applied";
    if applied {
        return Ok(0);
    }
    ensure!(
        operation.journal.phase != "rolling-back",
        "rollback is in progress; resume upgrade rollback"
    );
    for name in ordered(&operation) {
        let change = &operation.plan.files[&name];
        let current = storage::state(root, &name)?;
        ensure!(
            current == change.before || current == change.after,
            "file changed during upgrade: {name}"
        );
        let write = current != change.after;
        if write {
            storage::replace(root, &change.after, &operation.directory)?;
        }
        let newly_completed = !operation.journal.completed.contains(&name);
        if newly_completed {
            operation.journal.completed.push(name);
            operation.save()?;
        }
    }
    verify(root, &mut operation)
}
fn ordered(operation: &Operation) -> Vec<String> {
    let mut names: Vec<String> = operation
        .plan
        .files
        .iter()
        .filter(|(_, change)| change.before != change.after)
        .map(|(name, _)| name.clone())
        .collect();
    let binary = format!("{}/bin/agentrig", operation.plan.service);
    let manifest = format!("{}/manifest.json", operation.plan.service);
    names.sort_by_key(|name| match name.as_str() {
        name if name == binary => 0,
        config::FILE => 2,
        super::migration::LEGACY_FILE => 3,
        name if name == manifest => 4,
        _ => 1,
    });
    names
}
fn verify(root: &Path, operation: &mut Operation) -> Result<i32> {
    operation.phase("validating")?;
    for command in operation.plan.checks.clone() {
        let argv = vec![
            root.join(format!("{}/bin/agentrig", operation.plan.service))
                .to_string_lossy()
                .into_owned(),
            command.clone(),
            "--root".into(),
            root.to_string_lossy().into_owned(),
        ];
        let code = process::run(root, &argv, false)?;
        operation.journal.checks.insert(command, code);
        operation.save()?;
        let failed = code != 0;
        if failed {
            return Ok(code);
        }
    }
    operation.journal.next_action =
        "upgrade verified; upgrade rollback restores the previous installation".into();
    operation.phase("applied")?;
    println!("Upgrade applied and verified");
    Ok(0)
}
pub fn rollback(root: &Path) -> Result<i32> {
    let mut operation = Operation::open(root)?;
    operation.journal.next_action = "upgrade rollback resumes restoration".into();
    operation.phase("rolling-back")?;
    for name in ordered(&operation).into_iter().rev() {
        let change = &operation.plan.files[&name];
        let current = storage::state(root, &name)?;
        ensure!(
            current == change.before || current == change.after,
            "file changed after upgrade; preserve or resolve before rollback: {name}"
        );
        let restore = current != change.before;
        if restore {
            storage::replace(root, &change.before, &operation.directory)?;
        }
        let newly_restored = !operation.journal.restored.contains(&name);
        if newly_restored {
            operation.journal.restored.push(name);
            operation.save()?;
        }
    }
    operation.journal.next_action =
        "previous installation restored; prepare a new upgrade plan when ready".into();
    operation.phase("rolled-back")?;
    println!("Upgrade rolled back");
    Ok(0)
}
