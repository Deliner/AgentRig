use super::{Dependency, Issue, Settings};
use crate::structural_lint::languages::ENTRY_FILES;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

type Adjacency<'a> = BTreeMap<PathBuf, BTreeMap<PathBuf, &'a Dependency>>;

pub(super) fn cycles(dependencies: &[Dependency]) -> Vec<Issue> {
    let mut graph: Adjacency<'_> = BTreeMap::new();
    for edge in dependencies {
        let declaration = edge.relation.declares();
        if declaration {
            continue;
        }
        add_edge(&mut graph, edge);
    }
    Traversal::issues(graph, "directory")
}

// DECISION: D225
pub(super) fn file_cycles(dependencies: &[Dependency], selected: &BTreeSet<&Path>) -> Vec<Issue> {
    let mut graph: Adjacency<'_> = BTreeMap::new();
    for edge in dependencies {
        let Some(directory) = edge.source.parent() else {
            continue;
        };
        let sibling = edge.target.parent() == Some(directory) && edge.source != edge.target;
        let between_parts = !entry(&edge.source) && !entry(&edge.target);
        let used =
            !edge.relation.declares() && selected.contains(directory_key(directory).as_path());
        let counted = sibling && between_parts && used;
        if counted {
            let targets = graph.entry(edge.source.clone()).or_default();
            targets.entry(edge.target.clone()).or_insert(edge);
        }
    }
    Traversal::issues(graph, "file")
}

// DECISION: D225
fn entry(file: &Path) -> bool {
    let name = file.file_name().and_then(|name| name.to_str());
    let rust = name.is_some_and(|name| ENTRY_FILES.contains(&name));
    rust || file.file_stem().is_some_and(|stem| stem == "__init__")
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

// DECISION: D079
// DECISION: D142
// DECISION: D146
pub(super) fn directory_edges(
    dependencies: &[Dependency],
) -> impl Iterator<Item = (&Path, &Path, &Dependency)> {
    dependencies.iter().filter_map(|edge| {
        let (Some(source), Some(target)) = (edge.source.parent(), edge.target.parent()) else {
            return None;
        };
        let counted = !edge.relation.declares() && !edge.test && source != target;
        counted.then_some((source, target, edge))
    })
}

// DECISION: D142
pub(super) fn dependents(dependencies: &[Dependency]) -> Dependents<'_> {
    let mut dependents: Dependents<'_> = BTreeMap::new();
    for (source, target, edge) in directory_edges(dependencies) {
        dependents
            .entry(target)
            .or_default()
            .entry(source)
            .or_insert(edge);
    }
    dependents
}

pub(super) type Dependents<'a> = BTreeMap<&'a Path, BTreeMap<&'a Path, &'a Dependency>>;

// DECISION: D143
pub(super) fn neighbours(dependencies: &[Dependency], directory: &Path) -> Neighbours {
    let mut neighbours = Neighbours::default();
    for (source, target, edge) in directory_edges(dependencies) {
        let (source, target) = (directory_key(source), directory_key(target));
        let incoming = target == directory;
        let outgoing = source == directory;
        let side = if incoming {
            neighbours.incoming.entry(source)
        } else if outgoing {
            neighbours.outgoing.entry(target)
        } else {
            continue;
        };
        let edges = side.or_default();
        let named = evidence(edge);
        let fresh = !edges.contains(&named);
        if fresh {
            edges.push(named);
        }
    }
    neighbours
}

// DECISION: D143
#[derive(Default)]
pub struct Neighbours {
    pub incoming: BTreeMap<PathBuf, Vec<String>>,
    pub outgoing: BTreeMap<PathBuf, Vec<String>>,
}

// DECISION: D142
pub(super) fn evidence(edge: &Dependency) -> String {
    format!(
        "{source}:{line} uses {target}",
        source = edge.source.display(),
        line = edge.line,
        target = edge.target.display()
    )
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
        let foreign = module && !edge.relation.declares() && own != target;
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
                    "file uses {count} other project directories (budget {budget}): {listed}; split it by responsibility or move the wiring into a program entry point",
                    count = names.len(),
                    listed = names.join(", ")
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

pub(super) fn directory_key(path: &Path) -> PathBuf {
    let root = path.as_os_str().is_empty();
    if root {
        PathBuf::from(".")
    } else {
        path.to_owned()
    }
}

struct Traversal<'a> {
    graph: Adjacency<'a>,
    nodes: &'static str,
    finished: BTreeSet<PathBuf>,
    active: Vec<PathBuf>,
    issues: Vec<Issue>,
}

impl<'a> Traversal<'a> {
    fn issues(graph: Adjacency<'a>, nodes: &'static str) -> Vec<Issue> {
        let starts: Vec<_> = graph.keys().cloned().collect();
        let mut traversal = Self {
            graph,
            nodes,
            finished: BTreeSet::new(),
            active: Vec::new(),
            issues: Vec::new(),
        };
        for start in starts {
            traversal.visit(&start);
        }
        traversal.issues
    }

    fn visit(&mut self, node: &Path) {
        let finished = self.finished.contains(node);
        if finished {
            return;
        }
        self.active.push(node.into());
        let targets = self.graph.get(node).cloned().unwrap_or_default();
        for (target, edge) in targets {
            if let Some(index) = self.active.iter().position(|path| path == &target) {
                self.cycle(index, edge);
            } else {
                self.visit(&target);
            }
        }
        self.active.pop();
        self.finished.insert(node.into());
    }

    // DECISION: D142
    fn cycle(&mut self, index: usize, edge: &'a Dependency) {
        let cycle = &self.active[index..];
        let mut hops: Vec<&Dependency> = cycle
            .windows(2)
            .filter_map(|pair| self.graph.get(&pair[0])?.get(&pair[1]).copied())
            .collect();
        hops.push(edge);
        let mut paths: Vec<_> = cycle.iter().map(|p| p.display().to_string()).collect();
        paths.push(cycle[0].display().to_string());
        let named: Vec<_> = hops.into_iter().map(evidence).collect();
        self.issues.push(Issue::dependency(
            edge,
            format!(
                "{nodes} dependency cycle: {cycle}; {evidence}",
                nodes = self.nodes,
                cycle = paths.join(" -> "),
                evidence = named.join("; ")
            ),
        ));
    }
}
