use super::{Kind, Repository, git, mercurial};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeSet, fs, path::Path};

impl Repository<'_> {
    pub fn working_directories(&self, files: &[String]) -> Result<Vec<String>> {
        let known: BTreeSet<_> = files
            .iter()
            .flat_map(|file| Path::new(file).ancestors().skip(1))
            .collect();
        let mut output = Vec::new();
        walk(self, Path::new(""), &known, &mut output)?;
        output.sort();
        Ok(output)
    }
}

fn walk(
    repository: &Repository<'_>,
    relative: &Path,
    known: &BTreeSet<&Path>,
    output: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(repository.root.join(relative))? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let control = matches!(entry.file_name().to_str(), Some(".git" | ".hg"));
        let skip = control || !kind.is_dir();
        if skip {
            continue;
        }
        let path = relative.join(entry.file_name());
        let name = path.to_str().context("VCS directory path must be UTF-8")?;
        let hidden = !known.contains(path.as_path()) && ignored(repository, name)?;
        if hidden {
            continue;
        }
        output.push(name.into());
        walk(repository, &path, known, output)?;
    }
    Ok(())
}

fn ignored(repository: &Repository<'_>, path: &str) -> Result<bool> {
    match repository.kind {
        Kind::Git => {
            let output = git::command(
                repository.root,
                &["check-ignore", "--quiet", "--no-index", "--", path],
            )
            .output()?;
            ensure!(
                matches!(output.status.code(), Some(0 | 1)),
                "cannot inspect Git directory exclusion: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            Ok(output.status.success())
        }
        Kind::Mercurial => {
            let bytes = mercurial::run(repository.root, &["debugignore", "--", path])?;
            let output = String::from_utf8(bytes)?;
            let hidden = output.starts_with(&format!("{path} is ignored\n"));
            ensure!(
                hidden || output == format!("{path} is not ignored\n"),
                "unrecognized Mercurial directory exclusion response for {path}"
            );
            Ok(hidden)
        }
    }
}
