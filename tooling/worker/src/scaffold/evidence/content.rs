use anyhow::{Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub fn fingerprint(tree: &Path, origin: &Path, runtime: &str, staged: bool) -> Result<String> {
    let mut paths = paths(origin, staged)?;
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
fn paths(root: &Path, staged: bool) -> Result<Vec<PathBuf>> {
    let repository = root.join(".git").exists();
    if repository {
        let mut args = vec!["ls-files", "--cached", "-z"];
        let worktree = !staged;
        if worktree {
            args.extend(["--others", "--exclude-standard"]);
        }
        let output = Command::new("git").args(args).current_dir(root).output()?;
        ensure!(output.status.success(), "cannot inventory check inputs");
        return output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|name| !name.is_empty())
            .map(|name| Ok(PathBuf::from(String::from_utf8(name.to_vec())?)))
            .collect();
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

pub fn index(root: &Path) -> Result<String> {
    let entries = crate::util::git(root, &["ls-files", "--stage", "-z"])?;
    Ok(format!("{:x}", Sha256::digest(entries.as_bytes())))
}
