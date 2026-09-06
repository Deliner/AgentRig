use super::{Entry, FileKind};
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeMap, path::Path, process::Command};

fn command(root: &Path, args: &[&str]) -> Command {
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

pub(super) fn changes(root: &Path, base: &str, candidate: &str) -> Result<Vec<u8>> {
    run(
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
    )
}

pub(super) fn tree(root: &Path, revision: &str) -> Result<BTreeMap<String, Entry>> {
    let bytes = run(root, &["ls-tree", "-rz", "--full-tree", revision])?;
    let mut entries = BTreeMap::new();
    for entry in bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let (header, path) = std::str::from_utf8(entry)?
            .split_once('\t')
            .context("invalid Git tree entry")?;
        let fields: Vec<_> = header.split_whitespace().collect();
        ensure!(fields.len() == 3, "invalid Git tree entry");
        let kind = match fields[0] {
            "100644" => FileKind::File,
            "100755" => FileKind::Executable,
            "120000" => FileKind::Symlink,
            "160000" => FileKind::Submodule,
            _ => bail!("unsupported Git entry mode: {}", fields[0]),
        };
        entries.insert(
            path.into(),
            Entry {
                kind,
                object: fields[2].into(),
            },
        );
    }
    Ok(entries)
}
