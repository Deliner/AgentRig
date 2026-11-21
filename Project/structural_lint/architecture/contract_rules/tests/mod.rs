mod contracts;
mod coupling;
mod cycles;
mod features;
mod zones;

use super::{Dependency, Issue, Relation};
use std::{fs, path::Path};

fn write(root: &Path, directory: &str, source: &str) {
    let path = root.join(directory);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("architecture.yaml"), source).unwrap();
}

fn edge(source: &str, target: &str) -> Dependency {
    Dependency {
        source: source.into(),
        target: target.into(),
        line: 7,
        relation: Relation::Use,
        test: false,
    }
}

fn messages(issues: &[Issue]) -> Vec<&str> {
    issues.iter().map(|issue| issue.message.as_str()).collect()
}
