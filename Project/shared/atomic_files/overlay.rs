use crate::shared::fs_with_context::{self as fs, present};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Overlay<'a> {
    pub mirror: &'a Path,
    pub root: &'a Path,
}

struct Original {
    path: PathBuf,
    content: Option<Vec<u8>>,
}

pub struct Applied {
    mirror: PathBuf,
    originals: Vec<Original>,
}

impl Overlay<'_> {
    pub fn apply(&self) -> Result<Applied> {
        let mut applied = Applied {
            mirror: self.mirror.to_owned(),
            originals: Vec::new(),
        };
        for relative in files(self.mirror)? {
            let path = self.root.join(&relative);
            let content = present(fs::read(&path))?;
            let original = Original { path, content };
            let written = replace(&original, &self.mirror.join(&relative));
            applied.originals.push(original);
            if let Err(error) = written {
                applied.restore()?;
                return Err(error);
            }
        }
        Ok(applied)
    }
}

impl Applied {
    pub fn restore(self) -> Result<()> {
        for original in self.originals {
            match original.content {
                Some(content) => fs::write(&original.path, content)?,
                None => {
                    present(fs::remove_file(&original.path))?;
                }
            }
        }
        Ok(())
    }
    pub fn commit(self) -> Result<()> {
        let applied = !self.originals.is_empty();
        if applied {
            // DECISION: D213
            fs::remove_dir_all(&self.mirror)?;
        }
        Ok(())
    }
}

fn replace(original: &Original, source: &Path) -> Result<()> {
    let parent = original
        .path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("overlay path has no parent"))?;
    fs::create_dir_all(parent)?;
    fs::write(&original.path, fs::read(source)?)?;
    let created = original.content.is_none();
    if created {
        fs::set_permissions(&original.path, fs::metadata(source)?.permissions())?;
    }
    Ok(())
}

fn files(mirror: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let Some(entries) = present(fs::read_dir(mirror.join(&relative)))? else {
            continue;
        };
        for entry in entries {
            let entry = entry?;
            let path = relative.join(entry.file_name());
            let directory = entry
                .file_type()
                .map_err(|error| anyhow::anyhow!("{}: {error}", path.display()))?
                .is_dir();
            if directory {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}
