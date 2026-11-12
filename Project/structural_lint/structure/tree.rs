use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub fn render(directories: &BTreeMap<PathBuf, BTreeSet<String>>, path: &Path) -> String {
    let mut lines = vec![path.display().to_string()];
    branch(directories, path, "", &mut lines);
    lines.join("\n")
}

fn branch(
    directories: &BTreeMap<PathBuf, BTreeSet<String>>,
    directory: &Path,
    indent: &str,
    lines: &mut Vec<String>,
) {
    let Some(entries) = directories.get(directory) else {
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
        branch(
            directories,
            &child,
            &format!("{indent}{continuation}"),
            lines,
        );
    }
}
