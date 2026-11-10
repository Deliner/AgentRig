use crate::project_config::Context as Project;
use crate::shared::fs::{self, PermissionsExt};
use crate::shared::vcs::Repository;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

// DECISION: D062
#[derive(Clone, Copy)]
pub enum Tree {
    Staged,
    Working,
}
impl Tree {
    fn staged(self) -> bool {
        match self {
            Self::Staged => true,
            Self::Working => false,
        }
    }
}

pub fn fingerprint(context: &Project, origin: &Path, tree: Tree) -> Result<String> {
    fingerprint_paths(
        &context.root,
        &context.config.paths.runtime,
        paths(origin, tree)?,
    )
}

pub fn revision_fingerprint(context: &Project, origin: &Path, revision: &str) -> Result<String> {
    let repository = crate::shared::vcs::Repository::discover(origin)?
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
        feed(&mut digest, path.as_os_str().as_encoded_bytes())?;
        hash_file(&mut digest, &tree.join(path))?;
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn feed(digest: &mut Sha256, bytes: &[u8]) -> Result<()> {
    digest.update(u64::try_from(bytes.len())?.to_le_bytes());
    digest.update(bytes);
    Ok(())
}
fn hash_file(digest: &mut Sha256, path: &Path) -> Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.missing() => {
            feed(digest, b"absent")?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let symlink = meta.file_type().is_symlink();
    let file = meta.is_file();
    if symlink {
        feed(digest, b"symlink")?;
        feed(digest, fs::read_link(path)?.as_os_str().as_encoded_bytes())?;
    } else if file {
        feed(digest, &(meta.permissions().mode() & 0o111).to_le_bytes())?;
        feed(digest, &fs::read(path)?)?;
    } else {
        feed(digest, b"directory")?;
    }
    Ok(())
}
fn paths(root: &Path, tree: Tree) -> Result<Vec<PathBuf>> {
    if let Some(repository) = crate::shared::vcs::Repository::discover(root)? {
        let staged = tree.staged();
        let files = if staged {
            repository.staged_files()?
        } else {
            repository.working_files()?
        };
        return Ok(files.into_iter().map(PathBuf::from).collect());
    }
    let mut walk = Walk {
        root,
        paths: Vec::new(),
    };
    walk.visit(Path::new(""))?;
    Ok(walk.paths)
}
struct Walk<'a> {
    root: &'a Path,
    paths: Vec<PathBuf>,
}
impl Walk<'_> {
    fn visit(&mut self, parent: &Path) -> Result<()> {
        for entry in fs::read_dir(self.root.join(parent))? {
            let entry = entry?;
            let path = parent.join(entry.file_name());
            let directory = entry.file_type()?.is_dir();
            if directory {
                self.visit(&path)?;
            } else {
                self.paths.push(path);
            }
        }
        Ok(())
    }
}

pub fn native_index(root: &Path) -> Result<String> {
    let repository =
        Repository::discover(root)?.context("a VCS repository is required for --staged")?;
    Ok(format!("{:x}", Sha256::digest(repository.index_entries()?)))
}

pub fn index(root: &Path) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(Repository::new(root).index_entries()?)
    ))
}

pub fn head(root: &Path) -> Result<Option<String>> {
    Ok(crate::shared::vcs::Repository::discover(root)?
        .map(|repository| repository.head())
        .transpose()?
        .flatten())
}
