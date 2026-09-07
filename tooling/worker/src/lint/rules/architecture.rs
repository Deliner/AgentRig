use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, PathBuf},
};

#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub python_root: PathBuf,
    pub rust_roots: Vec<PathBuf>,
    pub rust_crates: BTreeMap<String, PathBuf>,
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
            "rust_crates": "local Rust crate name to a root listed in rust_roots; resolved to measured file edges before external classification",
            "external": "literal module prefixes by rust/python/javascript namespace; Rust defaults std/core/alloc",
            "resolution": {
                "rust": "Rust 2018+ declared module trees, local crate mappings and module-level aliases; Self and single inline trait bounds; observed prelude names with syntactic shadow checks; known macro/resource paths, standard/serde attributes and cfg(test); unsupported expansion and binding ambiguity remain incomplete",
                "python": "one import root, package initializers and namespace submodules; recognized loader aliases/values report incomplete binding analysis; package export ambiguity and runtime path changes unsupported",
                "javascript": "Node relative import/require; bare packages need explicit external declarations; loader values, module factory APIs and dynamic paths report incomplete analysis",
                "typescript": "relative bundler substitutions; no tsconfig aliases, NodeNext or typesVersions"
            },
            "incomplete_analysis": "reported at configured level; remaining semantic coverage is not a proof of absence"
        })
    }

    pub fn validate(&self) -> Result<()> {
        for path in std::iter::once(&self.python_root)
            .chain(&self.rust_roots)
            .chain(self.rust_crates.values())
        {
            let relative = path
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
            ensure!(
                relative,
                "architecture roots must be project-relative without parent traversal: {}",
                path.display()
            );
        }
        self.validate_crates()?;
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
    fn validate_crates(&self) -> Result<()> {
        for (name, root) in &self.rust_crates {
            let identifier = !name.is_empty()
                && name.chars().enumerate().all(|(index, c)| {
                    c == '_' || c.is_ascii_alphabetic() || (index > 0 && c.is_ascii_digit())
                });
            ensure!(
                identifier,
                "local Rust crate name must be an identifier: {name}"
            );
            ensure!(
                self.rust_roots.contains(root),
                "local Rust crate root must appear in rust_roots: {}",
                root.display()
            );
        }
        Ok(())
    }
}
