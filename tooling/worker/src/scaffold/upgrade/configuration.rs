use super::{
    model::{Action, Change, Plan},
    storage,
};
use crate::scaffold::{
    config,
    package::{
        self,
        manifest::{self, Manifest, Ownership},
    },
};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub fn create(root: &Path, path: &Path) -> Result<i32> {
    super::recovery::guard(root)?;
    let current = config::read(root)?;
    ensure!(
        current.runtime == config::VERSION,
        "update requires the installed runtime version"
    );
    let (configuration, mut files) = package::setup::update(root, path)?;
    let receipt = configuration.paths.service_path("manifest.json");
    let old: Manifest = serde_json::from_slice(&fs::read(root.join(&receipt))?)?;
    ensure!(
        old.package_version == current.runtime,
        "installation manifest version differs"
    );
    merge_codex(root, &mut files)?;
    let manifest = incoming(&files, &receipt)?;
    let directory = storage::directory(root)?;
    fs::create_dir_all(&directory)?;
    let pending = tempfile::Builder::new()
        .prefix("plan-")
        .tempdir_in(directory)?;
    let mut plan = Plan {
        version: 1,
        project: root.to_string_lossy().into_owned(),
        from_version: current.runtime,
        to_version: configuration.runtime,
        baseline: "configuration-inputs".into(),
        service: configuration.paths.service,
        files: BTreeMap::new(),
        manifest,
        checks: vec!["config-check".into(), "doctor".into(), "check".into()],
    };
    collect(root, pending.path(), (&old, &files), &mut plan)?;
    storage::json(&pending.path().join("plan.json"), &plan)?;
    super::plan::report(pending.path(), &plan)?;
    let directory = pending.keep();
    println!("Plan: {}", directory.join("plan.json").display());
    Ok(0)
}

fn incoming(files: &BTreeMap<String, Vec<u8>>, receipt: &str) -> Result<Manifest> {
    let mut manifest: Manifest = serde_json::from_slice(&files[receipt])?;
    for (path, entry) in &mut manifest.files {
        entry.sha256 = manifest::checksum(&files[path]);
    }
    Ok(manifest)
}

fn collect(
    root: &Path,
    directory: &Path,
    inputs: (&Manifest, &BTreeMap<String, Vec<u8>>),
    plan: &mut Plan,
) -> Result<()> {
    let (old, files) = inputs;
    let paths: BTreeSet<_> = old.files.keys().chain(files.keys()).collect();
    for path in paths {
        let before = storage::state(root, path)?;
        if let Some(hash) = &before.sha256 {
            ensure!(
                storage::blob(directory, &fs::read(root.join(&before.resolved))?)? == *hash,
                "input changed during planning: {path}"
            );
        }
        let memory = old
            .files
            .get(path)
            .is_some_and(|entry| entry.ownership == Ownership::Memory);
        let mut change = Change {
            before: before.clone(),
            after: before,
            action: Action::Keep,
            reason: "preserve project memory".into(),
            resolution: None,
        };
        let receipt = path == &format!("{}/manifest.json", plan.service);
        let resource = !receipt;
        if memory {
            plan.manifest
                .files
                .insert(path.clone(), old.files[path].clone());
        } else if resource {
            replacement(directory, path, (old, files), (&plan.manifest, &mut change))?;
        }
        plan.files.insert(path.clone(), change);
    }
    Ok(())
}

fn replacement(
    directory: &Path,
    path: &str,
    inputs: (&Manifest, &BTreeMap<String, Vec<u8>>),
    output: (&Manifest, &mut Change),
) -> Result<()> {
    let (old, files) = inputs;
    let (manifest, change) = output;
    let entry = manifest.files.get(path);
    change.after.sha256 = files
        .get(path)
        .map(|bytes| storage::blob(directory, bytes))
        .transpose()?;
    change.after.mode = change.after.sha256.as_ref().map(|_| {
        let executable = entry.is_some_and(|entry| entry.executable);
        let default = if executable { 0o755 } else { 0o644 };
        change
            .before
            .mode
            .filter(|mode| (mode & 0o111 != 0) == executable)
            .unwrap_or(default)
    });
    let modified = modified(old, path, &change.before);
    let identical = change.before == change.after;
    let removed = change.after.sha256.is_none();
    change.action = if identical {
        Action::Keep
    } else if modified {
        Action::Conflict
    } else if removed {
        Action::Remove
    } else {
        Action::Replace
    };
    change.reason = "update configured inputs; resolve local changes with keep or replace".into();
    Ok(())
}

fn modified(old: &Manifest, path: &str, before: &storage::State) -> bool {
    let expected = old
        .local
        .get(path)
        .cloned()
        .unwrap_or_else(|| old.files.get(path).map(|entry| entry.sha256.clone()));
    let permissions = old.files.get(path).is_none_or(|entry| {
        before
            .mode
            .is_some_and(|mode| (mode & 0o111 != 0) == entry.executable)
    });
    before.sha256 != expected || !permissions
}

fn merge_codex(root: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    let path = ".codex/config.toml";
    let mut existing: toml_edit::DocumentMut = fs::read_to_string(root.join(path))?.parse()?;
    let desired: toml_edit::DocumentMut = std::str::from_utf8(&files[path])?.parse()?;
    ensure!(
        existing
            .get("mcp_servers")
            .is_none_or(toml_edit::Item::is_table_like),
        "mcp_servers must be a table; existing settings preserved"
    );
    for name in ["worker_review", "worker_delegation"] {
        ensure!(
            existing
                .get("mcp_servers")
                .and_then(|servers| servers.get(name))
                .is_none_or(toml_edit::Item::is_table_like),
            "mcp_servers.{name} must be a table; existing settings preserved"
        );
        let server = desired
            .get("mcp_servers")
            .and_then(|servers| servers.get(name));
        if let Some(server) = server {
            for field in ["enabled", "command", "args", "tool_timeout_sec", "env_vars"] {
                existing["mcp_servers"][name][field] = server[field].clone();
            }
        } else if let Some(server) = existing
            .get_mut("mcp_servers")
            .and_then(|servers| servers.get_mut(name))
        {
            server["enabled"] = toml_edit::value(false);
        }
    }
    files.insert(path.into(), existing.to_string().into_bytes());
    Ok(())
}
