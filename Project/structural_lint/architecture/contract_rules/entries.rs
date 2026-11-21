use super::graph::{directory_key, entry};
use super::{dependency::Dependency, issue::Issue};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

// DECISION: D227
pub fn own_uses(dependencies: &[Dependency], selected: &BTreeSet<&Path>) -> Vec<Issue> {
    let own = dependencies.iter().filter(|edge| own_use(edge, selected));
    own.map(|edge| {
        let message = format!(
            "uses the entry {entry} of its own directory or a directory above; use the file that owns the item, and move an item the entry defines into a file named by its meaning",
            entry = edge.target.display()
        );
        Issue::dependency(edge, message)
    })
    .collect()
}

// DECISION: D227
fn own_use(edge: &Dependency, selected: &BTreeSet<&Path>) -> bool {
    let (Some(directory), Some(owner)) = (edge.source.parent(), edge.target.parent()) else {
        return false;
    };
    let working = edge.relation.uses() && !edge.test && edge.source != edge.target;
    let inside = directory.starts_with(owner) && entry(&edge.target);
    working && inside && selected.contains(directory_key(directory).as_path())
}

// DECISION: D228
pub struct Boundaries<'a> {
    groups: &'a BTreeSet<PathBuf>,
    entries: BTreeMap<&'a Path, Vec<&'a Path>>,
}

impl<'a> Boundaries<'a> {
    pub fn new(groups: &'a BTreeSet<PathBuf>, files: &'a [PathBuf]) -> Self {
        let mut entries: BTreeMap<&Path, Vec<&Path>> = BTreeMap::new();
        for file in files.iter().filter(|file| entry(file)) {
            if let Some(directory) = file.parent() {
                entries.entry(directory).or_default().push(file);
            }
        }
        Self { groups, entries }
    }

    // DECISION: D228
    pub fn bypasses(&self, dependencies: &[Dependency], selected: &BTreeSet<&Path>) -> Vec<Issue> {
        let mut issues = Vec::new();
        for edge in dependencies {
            let Some(directory) = edge.source.parent() else {
                continue;
            };
            let counted = edge.relation.uses() && module(&edge.target);
            let measured = counted && selected.contains(directory_key(directory).as_path());
            let crossed = measured.then(|| self.crossed(edge));
            if let Some(crossed) = crossed.flatten() {
                issues.push(Issue::dependency(edge, self.message(edge, &crossed)));
            }
        }
        issues
    }

    // DECISION: D228
    fn crossed(&self, edge: &Dependency) -> Option<PathBuf> {
        let source = edge.source.parent()?;
        let target = edge.target.as_path();
        let owner = target.parent()?;
        let common = owner.ancestors().find(|path| source.starts_with(path))?;
        let mut boundary = common.to_path_buf();
        loop {
            let inside = owner == boundary.as_path();
            if inside {
                return None;
            }
            let part = owner.strip_prefix(&boundary).ok()?.components().next()?;
            let next = boundary.join(part);
            let group = self.groups.contains(&next);
            if group {
                boundary = next;
                continue;
            }
            let through_entry = owner == next.as_path() && entry(target);
            return (!through_entry).then_some(next);
        }
    }

    fn message(&self, edge: &Dependency, crossed: &Path) -> String {
        let target = edge.target.display();
        let directory = crossed.display();
        let Some(entries) = self.entries.get(crossed) else {
            return format!(
                "uses {target} inside {directory}, which has no entry mod.rs, lib.rs, main.rs or __init__.py; nothing outside the directory may use its files, so give it an entry or move the used item to its owner"
            );
        };
        let named: Vec<_> = entries
            .iter()
            .map(|file| file.display().to_string())
            .collect();
        format!(
            "uses {target} inside {directory} past its entry {entry}; go through the entry",
            entry = named.join(" or ")
        )
    }
}

// DECISION: D228
fn module(file: &Path) -> bool {
    let extension = file.extension().and_then(|extension| extension.to_str());
    matches!(extension, Some("rs" | "py" | "pyi"))
}
