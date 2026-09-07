use super::{Issue, policy::Declaration};
use crate::lint::inventory::Inventory;
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn check(
    root: &Path,
    directory: &Path,
    declaration: &Declaration,
    inventory: &Inventory,
) -> Vec<Issue> {
    let mut messages = Vec::new();
    for (entries, kind) in [
        (&declaration.files, false),
        (&declaration.directories, true),
    ] {
        validate(&root.join(directory), entries, kind, &mut messages);
    }
    messages.extend(missing(directory, declaration, inventory));
    messages
        .into_iter()
        .map(|message| Issue {
            path: directory.join(super::policy::FILENAME),
            line: None,
            message,
        })
        .collect()
}

fn missing(directory: &Path, declaration: &Declaration, inventory: &Inventory) -> Vec<String> {
    let mut messages = Vec::new();
    for name in inventory.directories.get(directory).into_iter().flatten() {
        let path = directory.join(name);
        let path = path.strip_prefix(".").unwrap_or(&path);
        let child_directory = inventory.directories.contains_key(path);
        let entries = if child_directory {
            &declaration.directories
        } else {
            &declaration.files
        };
        let missing = !entries.contains_key(name);
        if missing {
            let block = if child_directory {
                "directories"
            } else {
                "files"
            };
            messages.push(format!("missing {block} entry: {name}"));
        }
    }
    messages
}

fn validate(
    directory: &Path,
    entries: &BTreeMap<String, String>,
    child_directory: bool,
    messages: &mut Vec<String>,
) {
    for (name, description) in entries {
        let immediate = Path::new(name)
            .file_name()
            .is_some_and(|part| part == name.as_str());
        let invalid = !immediate || description.trim().is_empty();
        if invalid {
            messages.push(format!("invalid inventory entry {name:?}: expected an immediate name and nonempty responsibility"));
            continue;
        }
        let metadata = fs::symlink_metadata(directory.join(name));
        let correct_kind = metadata.is_ok_and(|metadata| {
            if child_directory {
                metadata.is_dir()
            } else {
                metadata.is_file()
            }
        });
        let stale = !correct_kind;
        if stale {
            messages.push(format!(
                "inventory entry {name:?} is missing or has the wrong file/directory kind"
            ));
        }
    }
}
