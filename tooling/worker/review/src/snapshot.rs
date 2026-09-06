use crate::{
    config::{Repository, globs},
    digest,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
    process::Command,
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
    let output = Command::new("git")
        .current_dir(root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}
pub fn resolve(root: &Path, reference: &str) -> Result<String> {
    let bytes = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ],
    )?;
    Ok(String::from_utf8(bytes)?.trim().into())
}
pub fn prepare(
    root: &Path,
    revisions: (&str, &str),
    scope: &Repository,
    output: &Path,
) -> Result<Snapshot> {
    let base = resolve(root, revisions.0)?;
    let candidate = resolve(root, revisions.1)?;
    let allowed = globs(&scope.visible_paths)?;
    let changed = changed_paths(root, &base, &candidate)?;
    validate_changes(root, &changed, &allowed, &base)?;
    fs::create_dir(output).context("snapshot output must be new")?;
    let manifest = export(root, &candidate, &allowed, output)?;
    let contracts = contract_paths(scope, &manifest)?;
    let diff = diff(root, &base, &candidate)?;
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
    root: &Path,
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
    let old = tree(root, base)?;
    for path in changed {
        if let Some((mode, object)) = old.get(path) {
            inspect(root, path, mode, object)?;
        }
    }
    Ok(())
}
pub fn changed_paths(root: &Path, base: &str, candidate: &str) -> Result<Vec<String>> {
    let bytes = git(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--name-only",
            "-z",
            base,
            candidate,
            "--",
        ],
    )?;
    bytes
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| Ok(String::from_utf8(name.to_vec())?))
        .collect()
}
fn tree(root: &Path, commit: &str) -> Result<BTreeMap<String, (String, String)>> {
    let bytes = git(root, &["ls-tree", "-rz", "--full-tree", commit])?;
    let mut entries = BTreeMap::new();
    for entry in bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let text = std::str::from_utf8(entry)?;
        let (header, path) = text.split_once('\t').context("invalid Git tree entry")?;
        let fields: Vec<_> = header.split_whitespace().collect();
        ensure!(fields.len() == 3, "invalid Git tree entry");
        entries.insert(path.into(), (fields[0].into(), fields[2].into()));
    }
    Ok(entries)
}
fn export(
    root: &Path,
    commit: &str,
    allowed: &globset::GlobSet,
    output: &Path,
) -> Result<BTreeMap<String, Entry>> {
    let mut manifest = BTreeMap::new();
    for (path, (mode, object)) in tree(root, commit)? {
        let visible = allowed.is_match(&path);
        if visible {
            let bytes = inspect(root, &path, &mode, &object)?;
            let target = output.join(&path);
            fs::create_dir_all(target.parent().context("file parent required")?)?;
            fs::write(&target, &bytes)?;
            use std::os::unix::fs::PermissionsExt;
            let executable = mode == "100755";
            fs::set_permissions(
                target,
                fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 }),
            )?;
            manifest.insert(
                path,
                Entry {
                    sha256: digest(&bytes),
                    object,
                    mode,
                },
            );
        }
    }
    Ok(manifest)
}
fn inspect(root: &Path, path: &str, mode: &str, object: &str) -> Result<Vec<u8>> {
    safe_path(path)?;
    ensure!(
        ["100644", "100755"].contains(&mode),
        "cannot expose symlink or submodule: {path}"
    );
    let bytes = git(root, &["cat-file", "blob", object])?;
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
    Ok(String::from_utf8(git(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--binary",
            "--no-renames",
            base,
            candidate,
            "--",
        ],
    )?)?)
}
pub fn check_boundary(root: &Path, base: &str, candidate: &str, scope: &Repository) -> Result<()> {
    let changed = changed_paths(root, base, candidate)?;
    validate_changes(root, &changed, &globs(&scope.visible_paths)?, base)
}
