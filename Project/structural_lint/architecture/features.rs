// DECISION: D044
use super::{
    Dependency, Issue,
    policy::{Declaration, FILENAME},
};
use crate::structural_lint::inventory::Inventory;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

const TESTS: &str = "tests";

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Flat,
    Layered,
    Hexagonal,
    Onion,
    Clean,
    Custom,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Layered => "layered",
            Self::Hexagonal => "hexagonal",
            Self::Onion => "onion",
            Self::Clean => "clean",
            Self::Custom => "custom",
        }
    }

    fn parts(self) -> Option<&'static [&'static str]> {
        match self {
            Self::Flat => Some(&[]),
            Self::Layered => Some(&["presentation", "application", "domain", "infrastructure"]),
            Self::Hexagonal => Some(&["adapters", "ports", "domain"]),
            Self::Onion => Some(&["infrastructure", "application", "domain"]),
            Self::Clean => Some(&["frameworks", "adapters", "use_cases", "entities"]),
            Self::Custom => None,
        }
    }
}

pub(super) fn check(
    root: Option<&Path>,
    declarations: &BTreeMap<PathBuf, Declaration>,
    inventory: &Inventory,
    edges: &[Dependency],
) -> Vec<Issue> {
    let mut features = Features {
        declarations,
        inventory,
        typed: BTreeMap::new(),
        issues: Vec::new(),
    };
    for feature in root.map(|root| features.nested(root)).unwrap_or_default() {
        features.visit(&feature);
    }
    for (directory, declaration) in declarations {
        let misplaced = declaration.kind.is_some() && !features.typed.contains_key(directory);
        if misplaced {
            features.contract(
                directory,
                "type belongs only to a feature without nested features under the feature root"
                    .into(),
            );
        }
    }
    let directions = edges.iter().filter_map(|edge| features.direction(edge));
    let mut issues: Vec<_> = directions.collect();
    issues.append(&mut features.issues);
    issues
}

struct Features<'a> {
    declarations: &'a BTreeMap<PathBuf, Declaration>,
    inventory: &'a Inventory,
    typed: BTreeMap<PathBuf, Kind>,
    issues: Vec<Issue>,
}

impl Features<'_> {
    fn visit(&mut self, feature: &Path) {
        let Some(declaration) = self.declarations.get(feature) else {
            return;
        };
        if let Some(kind) = declaration.kind {
            self.typed.insert(feature.into(), kind);
            self.parts(feature, kind);
            return;
        }
        let nested = self.nested(feature);
        let untyped_leaf = nested.is_empty();
        if untyped_leaf {
            self.contract(feature, "feature without nested features must declare type: flat, layered, hexagonal, onion, clean or custom".into());
        }
        for child in nested {
            self.visit(&child);
        }
    }

    fn parts(&mut self, feature: &Path, kind: Kind) {
        let Some(parts) = kind.parts() else {
            return;
        };
        let present = self.subdirectories(feature);
        for part in parts {
            let missing = !present.iter().any(|(name, _)| name == part);
            if missing {
                let message = format!("{} feature lacks part directory: {part}", kind.name());
                self.contract(feature, message);
            }
        }
        for (name, _) in present {
            let outside = name != TESTS && !parts.contains(&name.as_str());
            if outside {
                let message = format!(
                    "{} feature has a directory outside its parts: {name}",
                    kind.name()
                );
                self.contract(feature, message);
            }
        }
    }

    fn nested(&self, directory: &Path) -> Vec<PathBuf> {
        let subdirectories = self.subdirectories(directory).into_iter();
        subdirectories
            .filter(|(name, _)| name != TESTS)
            .map(|(_, path)| path)
            .collect()
    }

    fn subdirectories(&self, directory: &Path) -> Vec<(String, PathBuf)> {
        let names = self.inventory.directories.get(directory).into_iter();
        names
            .flatten()
            .map(|name| {
                let path = directory.join(name);
                (name.clone(), path.strip_prefix(".").unwrap_or(&path).into())
            })
            .filter(|(_, path)| self.inventory.directories.contains_key(path))
            .collect()
    }

    fn contract(&mut self, directory: &Path, message: String) {
        self.issues.push(Issue {
            path: directory.join(FILENAME),
            line: None,
            message,
        });
    }

    fn direction(&self, edge: &Dependency) -> Option<Issue> {
        let declared = edge.module_declaration;
        if declared {
            return None;
        }
        let (feature, kind) = edge
            .source
            .ancestors()
            .find_map(|path| self.typed.get_key_value(path))?;
        let parts = kind.parts().filter(|parts| !parts.is_empty())?;
        let source = Role::of(feature, &edge.source, parts)?;
        let target = Role::of(feature, &edge.target, parts)?;
        let allowed = match (source, target) {
            (Role::Tests, _) | (Role::Root, Role::Root | Role::Part(_)) => true,
            (_, Role::Tests) | (Role::Part(_), Role::Root) => false,
            (Role::Part(from), Role::Part(to)) => to >= from,
        };
        let message = format!(
            "{}: {} -> {} goes against the dependency direction",
            kind.name(),
            source.name(parts),
            target.name(parts)
        );
        (!allowed).then(|| Issue::dependency(edge, message))
    }
}

#[derive(Clone, Copy)]
enum Role {
    Root,
    Part(usize),
    Tests,
}

impl Role {
    fn of(feature: &Path, path: &Path, parts: &[&'static str]) -> Option<Self> {
        let mut components = path.strip_prefix(feature).ok()?.components();
        let Some(Component::Normal(first)) = components.next() else {
            return None;
        };
        let file = components.next().is_none();
        if file {
            return Some(Self::Root);
        }
        let name = first.to_str()?;
        let tests = name == TESTS;
        if tests {
            return Some(Self::Tests);
        }
        parts.iter().position(|part| *part == name).map(Self::Part)
    }

    fn name(self, parts: &[&'static str]) -> &'static str {
        match self {
            Self::Root => "feature root files",
            Self::Part(index) => parts[index],
            Self::Tests => TESTS,
        }
    }
}
