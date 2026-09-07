use super::super::config::Context as Project;
use anyhow::{Context, Result};
use review_runner::vcs::{Backend, Repository};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub fn fingerprint(context: &Project, origin: &Path, staged: bool) -> Result<String> {
    fingerprint_paths(
        &context.root,
        &context.config.paths.runtime,
        paths(context, origin, staged)?,
    )
}

pub fn revision_fingerprint(context: &Project, origin: &Path, revision: &str) -> Result<String> {
    let repository = context
        .config
        .vcs
        .backend
        .repository_source(origin)?
        .context("revision checking requires a VCS repository")?;
    let paths = repository
        .tree(revision)?
        .into_keys()
        .map(PathBuf::from)
        .collect();
    fingerprint_paths(&context.root, &context.config.paths.runtime, paths)
}

fn fingerprint_paths(tree: &Path, runtime: &str, mut paths: Vec<PathBuf>) -> Result<String> {
    paths.sort();
    paths.dedup();
    let mut digest = Sha256::new();
    for path in paths {
        let runtime_file = path.starts_with(runtime);
        if runtime_file {
            continue;
        }
        feed(&mut digest, path.as_os_str().as_encoded_bytes());
        hash_file(&mut digest, &tree.join(path))?;
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn feed(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}
fn hash_file(digest: &mut Sha256, path: &Path) -> Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            feed(digest, b"absent");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let symlink = meta.file_type().is_symlink();
    let file = meta.is_file();
    if symlink {
        feed(digest, b"symlink");
        feed(digest, fs::read_link(path)?.as_os_str().as_encoded_bytes());
    } else if file {
        feed(digest, &(meta.permissions().mode() & 0o111).to_le_bytes());
        feed(digest, &fs::read(path)?);
    } else {
        feed(digest, b"directory");
    }
    Ok(())
}
fn paths(context: &Project, root: &Path, staged: bool) -> Result<Vec<PathBuf>> {
    if let Some(repository) = context.config.vcs.backend.repository_source(root)? {
        let files = if staged {
            repository.staged_files()?
        } else {
            repository.working_files()?
        };
        return Ok(files.into_iter().map(PathBuf::from).collect());
    }
    let mut paths = Vec::new();
    walk(root, Path::new(""), &mut paths)?;
    Ok(paths)
}
fn walk(root: &Path, parent: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root.join(parent))? {
        let entry = entry?;
        let path = parent.join(entry.file_name());
        let directory = entry.file_type()?.is_dir();
        if directory {
            walk(root, &path, paths)?;
        } else {
            paths.push(path);
        }
    }
    Ok(())
}

// Bootstrap only: the staged declaration is unavailable until the native index is exported.
pub fn native_index(root: &Path) -> Result<String> {
    let repository =
        Repository::discover(root)?.context("a VCS repository is required for --staged")?;
    Ok(format!("{:x}", Sha256::digest(repository.index_entries()?)))
}

pub fn index(backend: &Backend, root: &Path) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(backend.source(root).index_entries()?)
    ))
}

pub fn head(context: &Project, root: &Path) -> Result<Option<String>> {
    Ok(context
        .config
        .vcs
        .backend
        .repository_source(root)?
        .map(|repository| repository.head())
        .transpose()?
        .flatten())
}
