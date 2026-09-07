use super::{Entry, FileKind};
use anyhow::{Context as _, Result, bail, ensure};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path, process::Command};

pub(super) fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    // Mercurial can refresh caches and dirstate during native read commands.
    let mut command = Command::new("bwrap");
    command.args([
        "--die-with-parent",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--",
        "hg",
    ]);
    execute(command, root, args).context("read Mercurial repository with bubblewrap")
}

pub(super) fn initialize(root: &Path, base: &str) -> Result<()> {
    execute(Command::new("hg"), root, &["init"])?;
    execute(Command::new("hg"), root, &["branch", "--", base])?;
    Ok(())
}

fn execute(mut command: Command, root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = command
        .current_dir(root)
        .env("HGPLAIN", "1")
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .env_remove("HGPLAINEXCEPT")
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "Mercurial failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

pub(super) fn resolve(root: &Path, reference: &str) -> Result<String> {
    let bytes = run(root, &["log", "--rev", reference, "--template", "{node}\n"])?;
    Ok(String::from_utf8(bytes)?.trim().into())
}

// Setup inspects the trusted local configuration, unlike immutable source reads.
pub(super) fn configuration(root: &Path, key: &str) -> Result<String> {
    let output = Command::new("hg")
        .current_dir(root)
        .env("HGPLAIN", "1")
        .env("HGRCPATH", "")
        .env_remove("HGRCSKIPREPO")
        .args(["config", key])
        .output()?;
    ensure!(
        output.status.success() || output.status.code() == Some(1),
        "cannot inspect Mercurial hook registration: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

pub(super) fn working_files(root: &Path) -> Result<Vec<u8>> {
    run(
        root,
        &[
            "status",
            "--clean",
            "--modified",
            "--added",
            "--unknown",
            "--no-status",
            "--print0",
        ],
    )
}

pub(super) fn parents(root: &Path, revision: &str) -> Result<Vec<String>> {
    let bytes = run(
        root,
        &[
            "log",
            "--rev",
            revision,
            "--template",
            "{p1node}\n{p2node}\n",
        ],
    )?;
    Ok(String::from_utf8(bytes)?
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}

pub(super) fn changes(root: &Path, base: &str, candidate: &str) -> Result<Vec<u8>> {
    run(
        root,
        &[
            "status",
            "--rev",
            base,
            "--rev",
            candidate,
            "--modified",
            "--added",
            "--removed",
            "--no-status",
            "--print0",
        ],
    )
}

#[derive(Deserialize)]
struct ManifestEntry {
    path: String,
    hash: String,
    mode: String,
    #[serde(rename = "type")]
    file_type: String,
}

pub(super) fn tree(root: &Path, revision: &str) -> Result<BTreeMap<String, Entry>> {
    let bytes = run(
        root,
        &[
            "manifest",
            "--rev",
            revision,
            "--debug",
            "--template",
            "json",
        ],
    )?;
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(&bytes)?;
    let mut entries = BTreeMap::new();
    for entry in manifest {
        let kind = match (entry.mode.as_str(), entry.file_type.as_str()) {
            ("644", "") => FileKind::File,
            ("755", "*") => FileKind::Executable,
            ("644", "@") => FileKind::Symlink,
            _ => bail!("unsupported Mercurial entry mode/type at {}", entry.path),
        };
        entries.insert(
            entry.path,
            Entry {
                kind,
                object: entry.hash,
            },
        );
    }
    Ok(entries)
}
