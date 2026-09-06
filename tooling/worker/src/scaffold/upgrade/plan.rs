use super::{
    external, migration,
    model::{Action, Change, Plan, State},
    release, storage,
};
use crate::scaffold::{
    config,
    package::manifest::{Manifest, Ownership},
};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

struct Draft<'a> {
    root: &'a Path,
    configuration: &'a config::Config,
    directory: &'a Path,
    exported: &'a Path,
    old: Manifest,
    resources: BTreeMap<String, Vec<u8>>,
    plan: Plan,
}

pub fn create(root: &Path, binary: &Path) -> Result<i32> {
    let (configuration, exported) = prepare_release(root, binary)?;
    let (old, baseline) = baseline(root, &configuration)?;
    ensure!(
        old.package_version == release::FROM,
        "installation manifest does not match project version"
    );
    let directory = storage::directory(root)?;
    fs::create_dir_all(&directory)?;
    let pending = tempfile::Builder::new()
        .prefix("plan-")
        .tempdir_in(directory)?;
    let plan = new_plan(root, exported.path(), baseline)?;
    let mut draft = Draft {
        root,
        configuration: &configuration,
        directory: pending.path(),
        exported: exported.path(),
        old,
        resources: migration::resources(root, &configuration)?,
        plan,
    };
    draft.collect()?;
    report(draft.directory, &draft.plan)?;
    storage::json(&pending.path().join("plan.json"), &draft.plan)?;
    let directory = pending.keep();
    println!("Plan: {}", directory.join("plan.json").display());
    Ok(0)
}
fn prepare_release(root: &Path, binary: &Path) -> Result<(config::Config, tempfile::TempDir)> {
    let configuration = migration::configuration(root)?;
    ensure!(
        configuration.runtime == release::FROM,
        "only {} -> {} is implemented",
        release::FROM,
        release::TO
    );
    let binary = binary.canonicalize()?;
    ensure!(
        release::version(&binary)? == release::TO,
        "release must be {}",
        release::TO
    );
    let exported = release::export(&binary, &configuration)?;
    Ok((configuration, exported))
}
fn new_plan(root: &Path, exported: &Path, baseline: String) -> Result<Plan> {
    Ok(Plan {
        version: 1,
        project: root.to_string_lossy().into_owned(),
        from_version: release::FROM.into(),
        to_version: release::TO.into(),
        baseline,
        service: ".worker".into(),
        files: Default::default(),
        manifest: release::manifest(exported)?,
        checks: vec!["config-check".into(), "doctor".into(), "check".into()],
    })
}
fn baseline(root: &Path, configuration: &config::Config) -> Result<(Manifest, String)> {
    let installed = root.join(super::migration::MANIFEST).is_file();
    if installed {
        return Ok((release::manifest(root)?, "manifest".into()));
    }
    let binary = root.join(".worker/bin/discipline-worker");
    ensure!(
        release::version(&binary)? == release::FROM,
        "cannot reconstruct baseline from installed runtime"
    );
    let stock = release::export(&binary, configuration)?;
    Ok((release::manifest(stock.path())?, "reconstructed".into()))
}
impl Draft<'_> {
    fn collect(&mut self) -> Result<()> {
        let mut paths: BTreeSet<String> = self
            .old
            .files
            .keys()
            .chain(self.plan.manifest.files.keys())
            .cloned()
            .collect();
        paths.insert(self.configuration.paths.lint.clone());
        paths.insert(release::lint_path(&self.configuration.paths.lint));
        paths.extend(self.resources.keys().cloned());
        paths.extend(self.resources.keys().map(|path| release::lint_path(path)));
        paths.extend(self.configuration.hooks.reminder.iter().cloned());
        for path in paths {
            let change = self.change(&path)?;
            self.plan.files.insert(path, change);
        }
        let receipt = storage::state(self.root, super::migration::MANIFEST)?;
        self.save_before(&receipt)?;
        self.plan.files.insert(
            super::migration::MANIFEST.into(),
            Change {
                before: receipt.clone(),
                after: receipt,
                action: Action::Keep,
                reason: "receipt replaced after explicit conflict resolutions".into(),
                resolution: None,
            },
        );
        Ok(())
    }
    fn change(&self, path: &str) -> Result<Change> {
        let before = storage::state(self.root, path)?;
        self.save_before(&before)?;
        let mut change = Change {
            before: before.clone(),
            after: before,
            action: Action::Keep,
            reason: "preserve project content".into(),
            resolution: None,
        };
        let migrated = self.migrate(path, &mut change)?;
        let replace_stock = !migrated && !self.preserve(path);
        if replace_stock {
            self.stock_change(path, &mut change)?;
        }
        Ok(change)
    }
    fn migrate(&self, path: &str, change: &mut Change) -> Result<bool> {
        let lint = &self.configuration.paths.lint;
        let bytes = match self.migration_bytes(path)? {
            Some(bytes) => bytes,
            None if path == lint
                || path == migration::LEGACY_FILE
                || self.resources.contains_key(path) =>
            {
                change.after.sha256 = None;
                change.after.mode = None;
                change.action = Action::Remove;
                change.reason = "replace legacy path with its YAML configuration".into();
                return Ok(true);
            }
            None => return Ok(false),
        };
        change.after.sha256 = Some(storage::blob(self.directory, &bytes)?);
        change.after.mode = Some(change.before.mode.unwrap_or(0o644));
        let collision = path != external::CODEX
            && path != lint
            && !self.resources.contains_key(path)
            && change.before.sha256.is_some();
        change.action = if collision {
            Action::Conflict
        } else {
            Action::Replace
        };
        let project = path == config::FILE;
        let codex = path == external::CODEX;
        change.reason = if codex {
            "migrate managed executable references; preserve other Codex settings and comments"
        } else if project {
            "convert project settings and runtime pin to YAML; original formatting/comments retained in reviewed preimage"
        } else {
            "convert configured resource to YAML; original formatting/comments retained in reviewed preimage; resolve destination conflicts explicitly"
        }.into();
        Ok(true)
    }
    fn migration_bytes(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let codex = path == external::CODEX;
        if codex {
            return external::codex(self.root);
        }
        let root_config = path == config::FILE;
        if root_config {
            return Ok(Some(release::migrated(&fs::read_to_string(
                self.root.join(migration::LEGACY_FILE),
            )?)?));
        }
        let lint = &self.configuration.paths.lint;
        let lint_target = path == release::lint_path(lint)
            && (self.configuration.capabilities.lint || self.root.join(lint).is_file());
        if lint_target {
            return Ok(Some(release::lint_yaml(self.root, lint)?));
        }
        Ok(self
            .resources
            .iter()
            .find(|(source, _)| release::lint_path(source) == path)
            .map(|(_, bytes)| bytes.clone()))
    }
    fn preserve(&self, path: &str) -> bool {
        let ownership = self
            .plan
            .manifest
            .files
            .get(path)
            .or_else(|| self.old.files.get(path))
            .map(|entry| &entry.ownership);
        matches!(
            ownership,
            Some(Ownership::Memory | Ownership::Configuration)
        ) || path == self.configuration.paths.lint
            || self.configuration.hooks.reminder.as_deref() == Some(path)
            || path.starts_with(&format!("{}/", self.configuration.paths.memory))
    }
    fn stock_change(&self, path: &str, change: &mut Change) -> Result<()> {
        change.after = self.target(path, &change.before)?;
        let old = self.old.files.get(path);
        let unchanged = old.is_some_and(|entry| {
            Some(&entry.sha256) == change.before.sha256.as_ref()
                && Some(entry.executable) == change.before.mode.map(|mode| mode & 0o111 != 0)
        });
        let fresh = old.is_none() && change.before.sha256.is_none();
        change.action = classify(change, unchanged || fresh);
        change.reason = match change.action {
            Action::Conflict => "local change: set resolution to keep or replace",
            _ => "install release content",
        }
        .into();
        Ok(())
    }
    fn target(&self, path: &str, before: &State) -> Result<State> {
        let mut state = before.clone();
        match self.plan.manifest.files.get(path) {
            Some(entry) => {
                let bytes = fs::read(self.exported.join(path))?;
                let hash = storage::blob(self.directory, &bytes)?;
                ensure!(
                    hash == entry.sha256,
                    "release manifest checksum mismatch: {path}"
                );
                state.sha256 = Some(hash);
                state.mode = Some(if entry.executable { 0o755 } else { 0o644 });
            }
            None => {
                state.sha256 = None;
                state.mode = None;
            }
        }
        Ok(state)
    }
    fn save_before(&self, state: &State) -> Result<()> {
        if let Some(expected) = &state.sha256 {
            let bytes = fs::read(self.root.join(&state.resolved))?;
            ensure!(
                storage::blob(self.directory, &bytes)? == *expected,
                "file changed during planning: {}",
                state.resolved
            );
        }
        Ok(())
    }
}
pub(super) fn report(directory: &Path, plan: &Plan) -> Result<()> {
    let mut report = format!(
        "Upgrade {} -> {} (baseline: {})\n",
        plan.from_version, plan.to_version, plan.baseline
    );
    for (path, change) in &plan.files {
        let action = serde_json::to_string(&change.action)?;
        report.push_str(&format!("{action} {path}: {}\n", change.reason));
        let changed = change.before != change.after;
        if changed {
            report.push_str(&diff(directory, change)?);
        }
    }
    report.push_str(&format!("Checks: {}\n", plan.checks.join(", ")));
    fs::write(directory.join("diff.txt"), &report)?;
    print!("{report}");
    Ok(())
}
fn diff(directory: &Path, change: &Change) -> Result<String> {
    let left = directory.join("before");
    let right = directory.join("after");
    for (path, state) in [(&left, &change.before), (&right, &change.after)] {
        let bytes = match &state.sha256 {
            Some(hash) => storage::payload(directory, hash)?,
            None => Vec::new(),
        };
        fs::write(path, bytes)?;
    }
    let output = Command::new("git")
        .args(["diff", "--no-index", "--no-ext-diff", "--no-color"])
        .arg(&left)
        .arg(&right)
        .output()?;
    ensure!(
        matches!(output.status.code(), Some(0 | 1)),
        "cannot produce upgrade diff"
    );
    fs::remove_file(left)?;
    fs::remove_file(right)?;
    Ok(format!(
        "mode {:?} -> {:?}\n{}",
        change.before.mode,
        change.after.mode,
        String::from_utf8_lossy(&output.stdout)
    ))
}
fn classify(change: &Change, stock: bool) -> Action {
    let equal = change.before == change.after;
    let removed = change.after.sha256.is_none();
    let modified = !stock;
    if modified {
        Action::Conflict
    } else if equal {
        Action::Keep
    } else if removed {
        Action::Remove
    } else {
        Action::Replace
    }
}
