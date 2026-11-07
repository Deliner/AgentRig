use super::{Dependency, Issue, Settings};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

type Adjacency<'a> = BTreeMap<PathBuf, BTreeMap<PathBuf, &'a Dependency>>;

pub(super) fn cycles(dependencies: &[Dependency]) -> Vec<Issue> {
    let mut graph: Adjacency<'_> = BTreeMap::new();
    for edge in dependencies {
        if edge.module_declaration {
            continue;
        }
        add_edge(&mut graph, edge);
    }
    let mut traversal = Traversal {
        graph,
        finished: BTreeSet::new(),
        active: Vec::new(),
        issues: Vec::new(),
    };
    for start in traversal.graph.keys().cloned().collect::<Vec<_>>() {
        traversal.visit(&start);
    }
    traversal.issues
}

fn add_edge<'a>(graph: &mut Adjacency<'a>, edge: &'a Dependency) {
    let (Some(source), Some(target)) = (edge.source.parent(), edge.target.parent()) else {
        return;
    };
    let same_directory = source == target;
    if same_directory {
        return;
    }
    let Some(common) = source.ancestors().find(|path| target.starts_with(path)) else {
        return;
    };
    let boundary = |path: &Path| {
        path.strip_prefix(common)
            .ok()
            .and_then(|rest| rest.components().next())
            .map(|part| common.join(part))
            .unwrap_or_else(|| common.to_owned())
    };
    for (source, target) in [(source, target), (&boundary(source), &boundary(target))] {
        graph
            .entry(directory_key(source))
            .or_default()
            .entry(directory_key(target))
            .or_insert(edge);
    }
}

// DECISION: D121
const FAN_OUT: usize = 10;

// DECISION: D121
pub(super) fn fan_out(dependencies: &[Dependency], settings: &Settings) -> Vec<Issue> {
    let budget = settings.fan_out.unwrap_or(FAN_OUT);
    let mut used: BTreeMap<&Path, BTreeSet<&Path>> = BTreeMap::new();
    for edge in dependencies {
        let (Some(own), Some(target)) = (edge.source.parent(), edge.target.parent()) else {
            continue;
        };
        let extension = edge
            .target
            .extension()
            .and_then(|extension| extension.to_str());
        let module = matches!(extension, Some("rs" | "py" | "pyi"));
        let foreign = module && !edge.module_declaration && own != target;
        if foreign {
            used.entry(&edge.source).or_default().insert(target);
        }
    }
    used.into_iter()
        .filter(|(file, directories)| directories.len() > budget && !entry_point(file, settings))
        .map(|(file, directories)| {
            let names: Vec<_> = directories.iter().map(|path| directory_key(path).display().to_string()).collect();
            Issue {
                path: file.to_owned(),
                line: None,
                message: format!(
                    "file uses {} other project directories (budget {budget}): {}; split it by responsibility or move the wiring into a program entry point",
                    names.len(),
                    names.join(", ")
                ),
            }
        })
        .collect()
}

// DECISION: D121
fn entry_point(file: &Path, settings: &Settings) -> bool {
    let python_main = file.file_name().is_some_and(|name| name == "__main__.py");
    python_main || settings.rust_roots.iter().any(|root| root == file)
}

fn directory_key(path: &Path) -> PathBuf {
    let root = path.as_os_str().is_empty();
    if root {
        PathBuf::from(".")
    } else {
        path.to_owned()
    }
}

struct Traversal<'a> {
    graph: Adjacency<'a>,
    finished: BTreeSet<PathBuf>,
    active: Vec<PathBuf>,
    issues: Vec<Issue>,
}

impl Traversal<'_> {
    fn visit(&mut self, directory: &Path) {
        let finished = self.finished.contains(directory);
        if finished {
            return;
        }
        self.active.push(directory.into());
        let targets = self.graph.get(directory).cloned().unwrap_or_default();
        for (target, edge) in targets {
            if let Some(index) = self.active.iter().position(|path| path == &target) {
                self.cycle(index, edge);
            } else {
                self.visit(&target);
            }
        }
        self.active.pop();
        self.finished.insert(directory.into());
    }

    fn cycle(&mut self, index: usize, edge: &Dependency) {
        let mut paths: Vec<_> = self.active[index..]
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        paths.push(self.active[index].display().to_string());
        self.issues.push(Issue::dependency(
            edge,
            format!("directory dependency cycle: {}", paths.join(" -> ")),
        ));
    }
}
