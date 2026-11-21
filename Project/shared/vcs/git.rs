use super::tree_entry::{Entry, FileKind};
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeMap, path::Path, process::Command};

#[derive(Clone, Copy)]
pub struct Revisions<'a> {
    pub from: &'a str,
    pub to: &'a str,
}

pub fn context_text(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    let failed = !output.status.success();
    if failed {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

pub(super) fn branch_name(value: &str) -> bool {
    let reserved = value.is_empty() || value == "HEAD" || value.starts_with('-');
    let malformed = value.ends_with('.') || value.contains("..") || value.contains("@{");
    let forbidden_byte = value
        .bytes()
        .any(|byte| byte <= 32 || byte == 127 || b"~^:?*[\\".contains(&byte));
    let valid_parts = value
        .split('/')
        .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"));
    let rejected = reserved || malformed || forbidden_byte;
    !rejected && valid_parts
}

pub(super) fn command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .args(args);
    command
}

pub fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = command(root, args).output()?;
    ensure!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

pub(super) fn ancestor(root: &Path, revisions: Revisions<'_>) -> Result<bool> {
    let arguments = ["merge-base", "--is-ancestor", revisions.from, revisions.to];
    let status = command(root, &arguments).status()?;
    ensure!(
        matches!(status.code(), Some(0 | 1)),
        "cannot inspect Git ancestry"
    );
    Ok(status.success())
}

// DECISION: D215
pub(super) fn fork(root: &Path, revisions: Revisions<'_>) -> Result<Option<String>> {
    let Revisions {
        from: base,
        to: tip,
    } = revisions;
    let branch = format!("refs/heads/{base}");
    let known = command(root, &["rev-parse", "--verify", "--quiet", &branch]).output()?;
    let absent = known.status.code() == Some(1);
    if absent {
        return Ok(None);
    }
    ensure!(known.status.success(), "cannot read the base branch {base}");
    let output = command(root, &["merge-base", &branch, tip]).output()?;
    let unrelated = output.status.code() == Some(1);
    if unrelated {
        return Ok(None);
    }
    ensure!(
        output.status.success(),
        "cannot find where {tip} forks from {base}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(Some(String::from_utf8(output.stdout)?.trim().into()))
}

pub(super) fn head(root: &Path) -> Result<Option<String>> {
    let output = command(root, &["rev-parse", "--verify", "--quiet", "HEAD"]).output()?;
    let unborn = output.status.code() == Some(1);
    if unborn {
        return Ok(None);
    }
    ensure!(
        output.status.success(),
        "cannot read Git HEAD: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(Some(String::from_utf8(output.stdout)?.trim().into()))
}

pub(super) fn resolve(root: &Path, reference: &str) -> Result<String> {
    let bytes = run(
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

pub(super) fn working_files(root: &Path) -> Result<Vec<u8>> {
    run(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )
}

pub(super) fn parents(root: &Path, revision: &str) -> Result<Vec<String>> {
    let bytes = run(root, &["rev-list", "--parents", "-n", "1", revision, "--"])?;
    Ok(String::from_utf8(bytes)?
        .split_whitespace()
        .skip(1)
        .map(str::to_owned)
        .collect())
}

pub(super) fn commit_context(root: &Path) -> Result<(String, bool)> {
    let branch = String::from_utf8(run(root, &["branch", "--show-current"])?)?;
    let merge = String::from_utf8(run(root, &["rev-parse", "--git-path", "MERGE_HEAD"])?)?;
    Ok((branch.trim().into(), root.join(merge.trim()).exists()))
}

pub(super) fn staged_files(root: &Path) -> Result<Vec<u8>> {
    run(root, &["ls-files", "--cached", "-z"])
}

pub(super) fn index_entries(root: &Path) -> Result<Vec<u8>> {
    run(root, &["ls-files", "--stage", "-z"])
}

pub(super) fn export_index(root: &Path) -> Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    let prefix = format!("--prefix={}/", directory.path().display());
    run(root, &["checkout-index", "--all", &prefix])?;
    Ok(directory)
}

pub(super) fn changes(root: &Path, revisions: Revisions<'_>) -> Result<Vec<u8>> {
    run(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--name-only",
            "-z",
            revisions.from,
            revisions.to,
            "--",
        ],
    )
}

pub(super) fn tree(root: &Path, revision: &str) -> Result<BTreeMap<String, Entry>> {
    let bytes = run(root, &["ls-tree", "-rz", "--full-tree", revision])?;
    let mut entries = BTreeMap::new();
    for record in records(&bytes) {
        let ([mode, _, object], path) = record?;
        let kind = kind(mode)?;
        let object: String = object.into();
        entries.insert(path.into(), Entry { kind, object });
    }
    Ok(entries)
}

pub(super) fn index(root: &Path) -> Result<BTreeMap<String, Entry>> {
    staged(&index_entries(root)?)
}

// DECISION: D213
pub(super) fn stageable(root: &Path, paths: &[String]) -> Result<BTreeMap<String, Entry>> {
    let directory = tempfile::tempdir()?;
    let index = directory.path().join("index");
    let run = |args: &[&str]| -> Result<Vec<u8>> {
        let output = command(root, args).env("GIT_INDEX_FILE", &index).output()?;
        ensure!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(output.stdout)
    };
    if let Some(head) = head(root)? {
        run(&["read-tree", &head])?;
    }
    let mut update = vec!["update-index", "--add", "--remove", "--"];
    update.extend(paths.iter().map(String::as_str));
    run(&update)?;
    staged(&run(&["ls-files", "--stage", "-z"])?)
}

fn staged(bytes: &[u8]) -> Result<BTreeMap<String, Entry>> {
    let mut entries = BTreeMap::new();
    for record in records(bytes) {
        let ([mode, object, stage], path) = record?;
        let merged = stage == "0";
        if merged {
            let kind = kind(mode)?;
            let object: String = object.into();
            entries.insert(path.into(), Entry { kind, object });
        }
    }
    Ok(entries)
}

fn records(bytes: &[u8]) -> impl Iterator<Item = Result<([&str; 3], &str)>> {
    let records = bytes.split(|byte| *byte == 0).filter(|r| !r.is_empty());
    records.map(|record| {
        let (header, path) = std::str::from_utf8(record)?
            .split_once('\t')
            .context("invalid Git entry")?;
        let fields: Vec<_> = header.split_whitespace().collect();
        let fields: [&str; 3] = fields.try_into().ok().context("invalid Git entry")?;
        Ok((fields, path))
    })
}

fn kind(mode: &str) -> Result<FileKind> {
    Ok(match mode {
        "100644" => FileKind::File,
        "100755" => FileKind::Executable,
        "120000" => FileKind::Symlink,
        "160000" => FileKind::Submodule,
        _ => bail!("unsupported Git entry mode: {mode}"),
    })
}
