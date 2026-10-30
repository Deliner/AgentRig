// DECISION: D044
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_root: Option<PathBuf>,
}

// DECISION: D076
pub const RUST_SEED: [&str; 3] = ["std", "core", "alloc"];

// DECISION: D076
pub const PYTHON_SEED: [&str; 1] = ["__future__"];

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct External {
    pub rust: BTreeSet<String>,
    pub python: BTreeSet<String>,
}

impl Default for External {
    fn default() -> Self {
        Self {
            rust: RUST_SEED.map(str::to_owned).into(),
            python: BTreeSet::new(),
        }
    }
}

impl Settings {
    pub fn describe() -> serde_json::Value {
        serde_json::json!({
            "contract": "architecture.yaml: single-line purpose, files and directories responsibility maps; type on a feature without nested features; every selected directory",
            "selection": "directory include/exclude; explicit extensions select immediate source files",
            "feature_root": "selected directory whose subdirectories are features; untyped features are composite, features without nested features declare flat, layered, hexagonal, onion, clean or custom with required parts and inward dependencies",
            "python_root": "one project-relative import root; default project root",
            "rust_roots": "explicit project-relative crate root files; required for selected Rust sources",
            "rust_crates": "local Rust crate name to a root listed in rust_roots; resolved to measured file edges before external classification",
            "external": "literal module prefixes by rust/python namespace; Rust defaults std/core/alloc",
            "resolution": {
                "rust": "Rust 2018+ declared module trees, literal path attributes on file-level external modules, local crate mappings and module-level aliases; Self and single inline trait bounds; observed prelude names with syntactic shadow checks; known macro/resource paths, standard/serde attributes and cfg over test and target platform predicates (all configurations measured); unsupported expansion and binding ambiguity remain incomplete",
                "python": "one import root, package initializers and namespace submodules; recognized loader aliases/values report incomplete binding analysis; package export ambiguity and runtime path changes unsupported"
            },
            "incomplete_analysis": "reported at configured level; remaining semantic coverage is not a proof of absence"
        })
    }

    pub fn validate(&self) -> Result<()> {
        for path in std::iter::once(&self.python_root)
            .chain(&self.rust_roots)
            .chain(self.rust_crates.values())
            .chain(&self.feature_root)
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
        for name in self.external.rust.iter().chain(&self.external.python) {
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
