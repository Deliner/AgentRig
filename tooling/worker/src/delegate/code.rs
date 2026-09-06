#[cfg(test)]
mod tests;

use anyhow::{Context, Result, ensure};
use review_runner::{config::globs, digest, snapshot::safe_path};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Workspace {
    directory: PathBuf,
}

impl Workspace {
    pub fn prepare(snapshot: &Path, directory: &Path) -> Result<Self> {
        fs::create_dir(directory).context("code workspace must be new")?;
        let workspace = Self {
            directory: directory.to_owned(),
        };
        fs::create_dir(workspace.project())?;
        for name in files(snapshot)? {
            let target = workspace.project().join(&name);
            fs::create_dir_all(target.parent().unwrap())?;
            fs::copy(snapshot.join(name), target)?;
        }
        workspace.git(&["init", "--template="])?;
        let tree = workspace.index()?;
        fs::write(directory.join("base-tree"), tree)?;
        Ok(workspace)
    }

    pub fn open(directory: &Path) -> Self {
        Self {
            directory: directory.to_owned(),
        }
    }

    pub fn project(&self) -> PathBuf {
        self.directory.join("project")
    }

    pub fn patch(&self, allowed: &[String]) -> Result<(Vec<u8>, Value)> {
        let base = fs::read_to_string(self.directory.join("base-tree"))?;
        let candidate = self.index()?;
        let changed = self.changed(&base, &candidate, allowed)?;
        ensure!(!changed.is_empty(), "code task produced no changes");
        let patch = self.git(&[
            "diff",
            "--binary",
            "--full-index",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            &base,
            &candidate,
            "--",
        ])?;
        let report = json!({"snapshot_tree":base,"candidate_tree":candidate,"changed_paths":changed,
            "patch_sha256":digest(&patch),"patch_bytes":patch.len()});
        Ok((patch, report))
    }

    fn changed(&self, base: &str, candidate: &str, allowed: &[String]) -> Result<Vec<String>> {
        let names = self.git(&[
            "diff",
            "--name-only",
            "--no-renames",
            "-z",
            base,
            candidate,
            "--",
        ])?;
        let selected = globs(allowed)?;
        let mut changed = Vec::new();
        for name in names
            .split(|byte| *byte == 0)
            .filter(|name| !name.is_empty())
        {
            let name = std::str::from_utf8(name)?;
            ensure!(
                selected.is_match(name),
                "change outside write_paths: {name}"
            );
            changed.push(name.to_owned());
        }
        Ok(changed)
    }

    fn index(&self) -> Result<String> {
        self.git(&["read-tree", "--empty"])?;
        for name in files(&self.project())? {
            let path = self.project().join(&name);
            let object = self.git(&[
                "hash-object",
                "-w",
                "--no-filters",
                "--",
                path.to_str().context("invalid path")?,
            ])?;
            let object = std::str::from_utf8(&object)?.trim();
            let executable = fs::metadata(path)?.permissions().mode() & 0o111 != 0;
            let mode = if executable { "100755" } else { "100644" };
            self.git(&["update-index", "--add", "--cacheinfo", mode, object, &name])?;
        }
        Ok(String::from_utf8(self.git(&["write-tree"])?)?.trim().into())
    }

    fn git(&self, args: &[&str]) -> Result<Vec<u8>> {
        let output = Command::new("/usr/bin/git")
            .env_clear()
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .arg("--git-dir")
            .arg(self.directory.join("git"))
            .arg("--work-tree")
            .arg(self.project())
            .args(["-c", "core.attributesFile=/dev/null"])
            .args(args)
            .current_dir(&self.directory)
            .output()?;
        ensure!(
            output.status.success(),
            "isolated Git: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(output.stdout)
    }
}

fn files(root: &Path) -> Result<Vec<String>> {
    let mut pending = vec![root.to_owned()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            let name = path
                .strip_prefix(root)?
                .to_str()
                .context("invalid code path")?
                .to_owned();
            safe_path(&name)?;
            let kind = entry.file_type()?;
            ensure!(
                kind.is_file() || kind.is_dir(),
                "code path is not a regular file/directory: {name}"
            );
            let directory = kind.is_dir();
            if directory {
                pending.push(path);
            } else {
                files.push(name);
            }
        }
    }
    files.sort();
    Ok(files)
}
