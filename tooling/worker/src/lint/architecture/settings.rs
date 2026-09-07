use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Component, PathBuf},
};

#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub python_root: PathBuf,
    pub rust_roots: Vec<PathBuf>,
    pub external: External,
}

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct External {
    pub rust: BTreeSet<String>,
    pub python: BTreeSet<String>,
    pub javascript: BTreeSet<String>,
}

impl Default for External {
    fn default() -> Self {
        Self {
            rust: ["std", "core", "alloc"].map(str::to_owned).into(),
            python: BTreeSet::new(),
            javascript: BTreeSet::new(),
        }
    }
}

impl Settings {
    pub fn describe() -> serde_json::Value {
        serde_json::json!({
            "contract": "architecture.yaml: single-line purpose, files and directories responsibility maps, allow, deny, public; every selected directory",
            "selection": "directory include/exclude; explicit extensions select immediate source files; enclosing selected contracts also apply",
            "python_root": "one project-relative import root; default project root",
            "rust_roots": "explicit project-relative crate root files; required for selected Rust sources",
            "external": "literal module prefixes by rust/python/javascript namespace; Rust defaults std/core/alloc",
            "resolution": {
                "rust": "Rust 2018+ declared module trees and module-level aliases; macro/conditional attributes report incomplete expansion; no full lexical/wildcard binding resolution",
                "python": "one import root, package initializers and namespace submodules; recognized loader aliases/values report incomplete binding analysis; package export ambiguity and runtime path changes unsupported",
                "javascript": "Node relative import/require; bare packages need explicit external declarations; loader values, module factory APIs and dynamic paths report incomplete analysis",
                "typescript": "relative bundler substitutions; no tsconfig aliases, NodeNext or typesVersions"
            },
            "incomplete_analysis": "reported at configured level; remaining semantic coverage is not a proof of absence"
        })
    }

    pub fn validate(&self) -> Result<()> {
        for path in std::iter::once(&self.python_root).chain(&self.rust_roots) {
            let relative = path
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
            ensure!(
                relative,
                "architecture roots must be project-relative without parent traversal: {}",
                path.display()
            );
        }
        for name in self
            .external
            .rust
            .iter()
            .chain(&self.external.python)
            .chain(&self.external.javascript)
        {
            let literal = !name.is_empty() && !name.contains(['*', '?', '[', ']', '\\']);
            ensure!(
                literal,
                "architecture external modules must be literal names: {name}"
            );
        }
        Ok(())
    }
}
