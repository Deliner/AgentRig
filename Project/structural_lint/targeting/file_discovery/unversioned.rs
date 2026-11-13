use super::{Inventory, directories};
use crate::shared::fs_with_context as fs;
use anyhow::Result;
use globset::GlobSet;
use std::{collections::BTreeMap, path::Path};

pub fn unversioned(root: &Path, exclude: &GlobSet) -> Result<Inventory> {
    let mut inventory = Inventory {
        files: Vec::new(),
        directories: BTreeMap::new(),
    };
    Walk { root, exclude }.visit(Path::new(""), &mut inventory)?;
    inventory.files.sort();
    for (directory, entries) in directories(&inventory.files) {
        inventory
            .directories
            .entry(directory)
            .or_default()
            .extend(entries);
    }
    Ok(inventory)
}

struct Walk<'a> {
    root: &'a Path,
    exclude: &'a GlobSet,
}

impl Walk<'_> {
    fn visit(&self, relative: &Path, inventory: &mut Inventory) -> Result<()> {
        let at_root = relative.as_os_str().is_empty();
        let parent = if at_root { Path::new(".") } else { relative };
        inventory.directories.entry(parent.into()).or_default();
        for entry in fs::read_dir(self.root.join(relative))? {
            let entry = entry?;
            let path = relative.join(entry.file_name());
            let ignored = self.exclude.is_match(&path) || entry.file_type()?.is_symlink();
            if ignored {
                continue;
            }
            let kind = entry.file_type()?;
            let directory = kind.is_dir();
            let file = kind.is_file();
            if directory {
                inventory
                    .directories
                    .entry(parent.into())
                    .or_default()
                    .insert(entry.file_name().to_string_lossy().into_owned());
                self.visit(&path, inventory)?;
            } else if file {
                inventory.files.push(path);
            }
        }
        Ok(())
    }
}
