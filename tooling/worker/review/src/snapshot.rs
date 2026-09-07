use crate::{
    config::{Repository, globs},
    digest,
    vcs::{FileKind, Source},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    pub sha256: String,
    pub object: String,
    pub mode: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Snapshot {
    pub base: String,
    pub candidate: String,
    pub manifest: BTreeMap<String, Entry>,
    pub changed_paths: Vec<String>,
    pub contract_paths: Vec<String>,
    pub diff: String,
}
pub fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    crate::vcs::git_command(root, args)
}
pub fn resolve(root: &Path, reference: &str) -> Result<String> {
    crate::vcs::Repository::new(root, crate::vcs::Kind::Git).resolve(reference)
}
pub fn prepare(
    root: &Path,
    revisions: (&str, &str),
    scope: &Repository,
    output: &Path,
) -> Result<Snapshot> {
    let source = scope.vcs.source(root);
    let base = source.resolve(revisions.0)?;
    let candidate = source.resolve(revisions.1)?;
    let allowed = globs(&scope.visible_paths)?;
    let changed = source.changed_paths(&base, &candidate)?;
    validate_changes(&source, &changed, &allowed, &base)?;
    fs::create_dir(output).context("snapshot output must be new")?;
    let manifest = export(&source, &candidate, &allowed, output)?;
    let contracts = contract_paths(scope, &manifest)?;
    let diff = source.diff(&base, &candidate)?;
    Ok(Snapshot {
        base,
        candidate,
        manifest,
        changed_paths: changed,
        contract_paths: contracts,
        diff,
    })
}
fn validate_changes(
    source: &Source<'_>,
    changed: &[String],
    allowed: &globset::GlobSet,
    base: &str,
) -> Result<()> {
    for path in changed {
        ensure!(
            allowed.is_match(path),
            "change outside visible_paths: {path}"
        );
        safe_path(path)?;
    }
    let old = source.tree(base)?;
    for path in changed {
        if let Some(entry) = old.get(path) {
            inspect(source, base, path, entry.kind)?;
        }
    }
    Ok(())
}
pub fn changed_paths(root: &Path, base: &str, candidate: &str) -> Result<Vec<String>> {
    crate::vcs::Repository::new(root, crate::vcs::Kind::Git).changed_paths(base, candidate)
}
fn export(
    source: &Source<'_>,
    commit: &str,
    allowed: &globset::GlobSet,
    output: &Path,
) -> Result<BTreeMap<String, Entry>> {
    let mut manifest = BTreeMap::new();
    for (path, entry) in source.tree(commit)? {
        let visible = allowed.is_match(&path);
        if visible {
            let bytes = inspect(source, commit, &path, entry.kind)?;
            let target = output.join(&path);
            fs::create_dir_all(target.parent().context("file parent required")?)?;
            fs::write(&target, &bytes)?;
            use std::os::unix::fs::PermissionsExt;
            let executable = entry.kind == FileKind::Executable;
            fs::set_permissions(
                target,
                fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 }),
            )?;
            manifest.insert(
                path,
                Entry {
                    sha256: digest(&bytes),
                    object: entry.object,
                    mode: entry.kind.mode().into(),
                },
            );
        }
    }
    Ok(manifest)
}
fn inspect(source: &Source<'_>, revision: &str, path: &str, kind: FileKind) -> Result<Vec<u8>> {
    safe_path(path)?;
    ensure!(
        matches!(kind, FileKind::File | FileKind::Executable),
        "cannot expose symlink or submodule: {path}"
    );
    let bytes = source.read(revision, path)?;
    let content = String::from_utf8_lossy(&bytes);
    let private_key = [
        "-----BEGIN PRIVATE KEY-----",
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----BEGIN OPENSSH PRIVATE KEY-----",
    ]
    .iter()
    .any(|marker| content.contains(marker));
    ensure!(
        !private_key,
        "private key material in selected file: {path}"
    );
    Ok(bytes)
}
pub fn safe_path(path: &str) -> Result<()> {
    for component in Path::new(path).components() {
        let Component::Normal(name) = component else {
            anyhow::bail!("unsafe project path: {path}");
        };
        let name = name.to_string_lossy();
        let sensitive = [
            ".git",
            ".hg",
            ".codex",
            ".agents",
            ".ssh",
            ".env",
            "auth.json",
            "credentials.json",
        ]
        .contains(&name.as_ref())
            || name.starts_with(".env.");
        ensure!(
            !sensitive,
            "secret or development-control path is not review input: {path}"
        );
    }
    Ok(())
}
fn contract_paths(scope: &Repository, manifest: &BTreeMap<String, Entry>) -> Result<Vec<String>> {
    let mut paths = std::collections::BTreeSet::new();
    for pattern in &scope.contract_paths {
        let matching = globs(std::slice::from_ref(pattern))?;
        let selected: Vec<_> = manifest
            .keys()
            .filter(|path| matching.is_match(path))
            .cloned()
            .collect();
        ensure!(
            !selected.is_empty(),
            "contract path is missing or outside visible_paths: {pattern}"
        );
        paths.extend(selected);
    }
    Ok(paths.into_iter().collect())
}

pub fn diff(root: &Path, base: &str, candidate: &str) -> Result<String> {
    crate::vcs::Repository::new(root, crate::vcs::Kind::Git).diff(base, candidate)
}
pub fn check_boundary(root: &Path, base: &str, candidate: &str, scope: &Repository) -> Result<()> {
    let source = scope.vcs.source(root);
    let changed = source.changed_paths(base, candidate)?;
    validate_changes(&source, &changed, &globs(&scope.visible_paths)?, base)
}
