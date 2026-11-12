use super::Location;
use super::inventory::{self, Inventory};
use crate::shared::console;
use crate::shared::paths::patterns::globs;
use anyhow::{Result, bail, ensure};
use std::path::{Path, PathBuf};

pub(super) fn run(location: Location<'_>, args: &[String]) -> Result<i32> {
    let [path] = args else {
        bail!("lint-tree PATH");
    };
    let path = super::explain::selected_path(path)?;
    let config = location.load()?;
    let inventory = inventory::collect(location.root, &globs(&config.exclude)?)?;
    ensure!(
        inventory.directories.contains_key(&path),
        "lint-tree PATH must name a directory of the lint inventory: {}",
        path.display()
    );
    let mut lines = vec![path.display().to_string()];
    branch(&inventory, &path, "", &mut lines);
    console::stdout(lines.join("\n"));
    Ok(0)
}

fn branch(inventory: &Inventory, directory: &Path, indent: &str, lines: &mut Vec<String>) {
    let Some(entries) = inventory.directories.get(directory) else {
        return;
    };
    let count = entries.len();
    let root = directory == Path::new(".");
    for (index, name) in entries.iter().enumerate() {
        let last = index + 1 == count;
        let (joint, continuation) = if last {
            ("└── ", "    ")
        } else {
            ("├── ", "│   ")
        };
        lines.push(format!("{indent}{joint}{name}"));
        let child = if root {
            PathBuf::from(name)
        } else {
            directory.join(name)
        };
        branch(inventory, &child, &format!("{indent}{continuation}"), lines);
    }
}
