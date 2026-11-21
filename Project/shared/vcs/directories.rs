use super::git;
use super::repository::Repository;
use crate::shared::fs_with_context as fs;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
};

impl Repository<'_> {
    pub fn working_directories(&self, files: &[String]) -> Result<Vec<String>> {
        let known: BTreeSet<_> = files
            .iter()
            .flat_map(|file| Path::new(file).ancestors().skip(1))
            .collect();
        let mut output = Vec::new();
        let mut round = vec![PathBuf::new()];
        while !round.is_empty() {
            let Walked {
                directories,
                unknown,
            } = walk(self, round, &known)?;
            output.extend(directories);
            round = unignored(self, unknown)?;
        }
        output.sort();
        Ok(output)
    }
}

struct Walked {
    directories: Vec<String>,
    unknown: Vec<PathBuf>,
}

fn walk(
    repository: &Repository<'_>,
    start: Vec<PathBuf>,
    known: &BTreeSet<&Path>,
) -> Result<Walked> {
    let mut walked = Walked {
        directories: Vec::new(),
        unknown: Vec::new(),
    };
    let mut pending = start;
    while let Some(directory) = pending.pop() {
        for path in subdirectories(repository, &directory)? {
            let listed = known.contains(path.as_path());
            match listed {
                true => pending.push(path),
                false => walked.unknown.push(path),
            }
        }
        let nested = !directory.as_os_str().is_empty();
        if nested {
            walked.directories.push(name(&directory)?.to_owned());
        }
    }
    Ok(walked)
}

fn subdirectories(repository: &Repository<'_>, relative: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(repository.root.join(relative))? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let control = entry.file_name() == ".git";
        let skip = control || !kind.is_dir();
        if skip {
            continue;
        }
        found.push(relative.join(entry.file_name()));
    }
    Ok(found)
}

fn unignored(repository: &Repository<'_>, unknown: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let none = unknown.is_empty();
    if none {
        return Ok(Vec::new());
    }
    let mut input = Vec::new();
    for path in &unknown {
        input.extend(name(path)?.as_bytes());
        input.push(0);
    }
    let output = ignored(repository, &input)?;
    let hidden: BTreeSet<&[u8]> = output.split(|byte| *byte == 0).collect();
    let visible = unknown
        .into_iter()
        .filter(|path| !hidden.contains(path.as_os_str().as_encoded_bytes()));
    Ok(visible.collect())
}

fn ignored(repository: &Repository<'_>, input: &[u8]) -> Result<Vec<u8>> {
    let arguments = ["check-ignore", "--no-index", "--stdin", "-z"];
    let mut child = git::command(repository.root, &arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut feed = child
        .stdin
        .take()
        .context("git check-ignore has no input")?;
    let (written, output) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || feed.write_all(input));
        let output = child.wait_with_output();
        (writer.join(), output)
    });
    let output = output?;
    ensure!(
        matches!(output.status.code(), Some(0 | 1)),
        "cannot inspect Git directory exclusion: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    written
        .map_err(|panic| anyhow::anyhow!("writing Git directory paths panicked: {panic:?}"))??;
    Ok(output.stdout)
}

fn name(path: &Path) -> Result<&str> {
    path.to_str().context("VCS directory path must be UTF-8")
}
