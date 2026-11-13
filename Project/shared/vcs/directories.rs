use super::{Repository, git};
use crate::shared::fs_with_context as fs;
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeSet, path::Path};

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
        let control = entry.file_name() == ".git";
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
