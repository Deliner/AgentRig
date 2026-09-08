use super::{Entry, FileKind};
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeMap, path::Path, process::Command};

/// Query Git using the invoking process environment, including hook repository context.
pub fn context_text(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    let failed = !output.status.success();
    if failed {
        bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

pub(super) fn branch_name(value: &str) -> bool {
    !value.is_empty()
        && value != "HEAD"
        && !value.starts_with('-')
        && !value.ends_with('.')
        && !value.contains("..")
        && !value.contains("@{")
        && !value
            .bytes()
            .any(|byte| byte <= 32 || byte == 127 || b"~^:?*[\\".contains(&byte))
        && value
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
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

pub(super) fn ancestor(root: &Path, older: &str, newer: &str) -> Result<bool> {
    let status = command(root, &["merge-base", "--is-ancestor", older, newer]).status()?;
    ensure!(
        matches!(status.code(), Some(0 | 1)),
        "cannot inspect Git ancestry"
    );
    Ok(status.success())
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

#[cfg(test)]
mod tests {
    use super::{context_text, run};
    use std::{path::Path, process::Command};

    fn initialize(root: &Path, branch: &str) {
        assert!(
            Command::new("git")
                .args(["init", "-q", "-b", branch])
                .arg(root)
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn query_inherits_repository_context() {
        if let Ok(root) = std::env::var("AGENTRIG_TEST_GIT_CONTEXT_ROOT") {
            let args = ["branch", "--show-current"];
            assert_eq!(context_text(Path::new(&root), &args).unwrap(), "inherited");
            assert_eq!(
                String::from_utf8(run(Path::new(&root), &args).unwrap())
                    .unwrap()
                    .trim(),
                "local"
            );
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let inherited = tempfile::tempdir().unwrap();
        for (path, branch) in [(root.path(), "local"), (inherited.path(), "inherited")] {
            initialize(path, branch);
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "vcs::git::tests::query_inherits_repository_context",
            ])
            .env("AGENTRIG_TEST_GIT_CONTEXT_ROOT", root.path())
            .env("GIT_DIR", inherited.path().join(".git"))
            .env("GIT_WORK_TREE", inherited.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn query_failure_preserves_git_stderr() {
        let root = tempfile::tempdir().unwrap();
        let args = ["--not-an-agentrig-option"];
        let expected = Command::new("git")
            .args(args)
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(!expected.status.success());
        assert_eq!(
            context_text(root.path(), &args).unwrap_err().to_string(),
            String::from_utf8_lossy(&expected.stderr)
        );
    }
}
