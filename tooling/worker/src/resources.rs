use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub struct File {
    pub bytes: Vec<u8>,
    pub executable: bool,
}

#[derive(Serialize)]
pub struct Input {
    pub sha256: String,
    pub executable: bool,
}

pub struct Bundle {
    prefix: String,
    pub files: BTreeMap<String, File>,
    pub inputs: BTreeMap<PathBuf, Input>,
}

impl Bundle {
    pub fn new(service: &str) -> Self {
        Self {
            prefix: format!("{service}/inputs"),
            files: BTreeMap::new(),
            inputs: BTreeMap::new(),
        }
    }

    pub fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let path = path
            .canonicalize()
            .with_context(|| format!("resource {}", path.display()))?;
        let metadata = fs::metadata(&path)?;
        ensure!(
            metadata.is_file(),
            "resource must be a file: {}",
            path.display()
        );
        let bytes = fs::read(&path)?;
        let input = Input {
            sha256: digest(&bytes),
            executable: metadata.permissions().mode() & 0o111 != 0,
        };
        if let Some(old) = self.inputs.get(&path) {
            ensure!(
                old.sha256 == input.sha256 && old.executable == input.executable,
                "resource changed while preparing: {}",
                path.display()
            );
        }
        self.inputs.insert(path, input);
        Ok(bytes)
    }

    pub fn copy(&mut self, path: &Path) -> Result<String> {
        let bytes = self.read(path)?;
        let executable = self.inputs[&path.canonicalize()?].executable;
        self.put(name(path)?, bytes, executable)
    }

    pub fn put(&mut self, name: &str, bytes: Vec<u8>, executable: bool) -> Result<String> {
        ensure!(
            Path::new(name).file_name().and_then(|name| name.to_str()) == Some(name),
            "resource name must be a filename"
        );
        let mut hash = Sha256::new();
        hash.update([u8::from(executable)]);
        hash.update(&bytes);
        let path = format!("{}/{:x}/{name}", self.prefix, hash.finalize());
        self.files.insert(path.clone(), File { bytes, executable });
        Ok(path)
    }

    pub fn directory(&mut self, path: &Path) -> Result<String> {
        let mut files = BTreeMap::new();
        self.collect(path, Path::new(""), &mut files)?;
        let descriptions: BTreeMap<_, _> = files
            .iter()
            .map(|(name, file)| (name, (digest(&file.bytes), file.executable)))
            .collect();
        let hash = digest(&serde_json::to_vec(&descriptions)?);
        let target = format!("{}/{hash}/{}", self.prefix, name(path)?);
        for (relative, file) in files {
            self.files.insert(format!("{target}/{relative}"), file);
        }
        Ok(target)
    }

    pub fn directory_at(&mut self, path: &Path, target: &str) -> Result<()> {
        let mut files = BTreeMap::new();
        self.collect(path, Path::new(""), &mut files)?;
        for (relative, file) in files {
            let name = format!("{target}/{relative}");
            if let Some(existing) = self.files.get(&name) {
                ensure!(
                    existing.bytes == file.bytes && existing.executable == file.executable,
                    "resource destination conflict: {name}"
                );
            }
            self.files.insert(name, file);
        }
        Ok(())
    }

    fn collect(
        &mut self,
        root: &Path,
        relative: &Path,
        files: &mut BTreeMap<String, File>,
    ) -> Result<()> {
        for entry in fs::read_dir(root.join(relative))? {
            let entry = entry?;
            let path = relative.join(entry.file_name());
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "resource directory contains a symlink: {}",
                entry.path().display()
            );
            let directory = kind.is_dir();
            if directory {
                self.collect(root, &path, files)?;
            } else {
                let bytes = self.read(&entry.path())?;
                let executable = self.inputs[&entry.path().canonicalize()?].executable;
                files.insert(
                    path.to_str().context("resource path must be UTF-8")?.into(),
                    File { bytes, executable },
                );
            }
        }
        Ok(())
    }

    pub fn sibling(&self, target: &str) -> Result<PathBuf> {
        let relative = target
            .strip_prefix(&format!("{}/", self.prefix))
            .context("resource is outside input bundle")?;
        Ok(Path::new("..").join(relative))
    }

    pub fn verify(&self) -> Result<()> {
        for (path, input) in &self.inputs {
            let executable = fs::metadata(path)?.permissions().mode() & 0o111 != 0;
            ensure!(
                digest(&fs::read(path)?) == input.sha256 && executable == input.executable,
                "resource changed before installation: {}",
                path.display()
            );
        }
        Ok(())
    }
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn name(path: &Path) -> Result<&str> {
    path.file_name()
        .and_then(|name| name.to_str())
        .context("resource filename must be UTF-8")
}
