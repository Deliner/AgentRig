use super::{Dependency, Issue};
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
    let (source, target) = (edge.source.parent().unwrap(), edge.target.parent().unwrap());
    let same_directory = source == target;
    if same_directory {
        return;
    }
    let common = source
        .ancestors()
        .find(|path| target.starts_with(path))
        .unwrap();
    let boundary = |path: &Path| {
        path.strip_prefix(common)
            .unwrap()
            .components()
            .next()
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
