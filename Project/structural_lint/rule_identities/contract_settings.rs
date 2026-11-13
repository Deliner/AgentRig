// DECISION: D044
use crate::shared::paths::glob_patterns::globs;
use anyhow::{Context, Result, ensure};
use globset::GlobSet;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, PathBuf},
};

pub const FILENAME: &str = "architecture.yaml";

#[derive(Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub python_root: PathBuf,
    pub rust_roots: Vec<PathBuf>,
    pub rust_crates: BTreeMap<String, PathBuf>,
    pub external: External,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_root: Option<PathBuf>,
    // DECISION: D105
    pub disallowed: Vec<Disallowed>,
    // DECISION: D121
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fan_out: Option<usize>,
}

// DECISION: D105
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Disallowed {
    pub path: String,
    pub reason: String,
    pub replacement: String,
    #[serde(default)]
    pub allow: Vec<String>,
    // DECISION: D125
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<String>,
}

// DECISION: D105
impl Disallowed {
    pub fn allowed(&self) -> Result<GlobSet> {
        globs(&self.allow).with_context(|| format!("disallowed path allow globs: {}", self.path))
    }
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

// DECISION: D104
// DECISION: D136
impl Settings {
    pub fn describe() -> serde_json::Value {
        serde_json::json!({
            "contract": "architecture.yaml: single-line purpose, files and directories responsibility maps; type on a feature without nested features; zone pure on a directory whose own files must not reach std::env, std::fs, std::process, read the wall clock (Instant::now, SystemTime::now), draw entropy (rand::random, rand::thread_rng, rand::rng, getrandom::getrandom, getrandom::fill), reach std::net, std::thread, std::io::stdin, std::io::stdout or std::io::stderr, or print through print!, println!, eprint! or eprintln!; dependents budget capping how many other directories may depend on a directory; every selected directory",
            "selection": "directory include/exclude; explicit extensions select immediate source files",
            "feature_root": "selected directory whose subdirectories are features; untyped features are composite, features without nested features declare flat, layered, hexagonal, onion, clean or custom with required parts and inward dependencies",
            "python_root": "one project-relative import root; default project root",
            "rust_roots": "explicit project-relative crate root files; required for selected Rust sources",
            "rust_crates": "local Rust crate name to a root listed in rust_roots; resolved to measured file edges before external classification",
            "external": "literal module prefixes by rust/python namespace; Rust defaults std/core/alloc",
            "disallowed": "resolved Rust external paths forbidden outside their owner: entries of path, reason, replacement and allow file globs; a reference resolved to the path or a :: subpath from a file outside allow is a finding naming the reason and replacement; optional methods lists method names, and a call of one without arguments whose result is propagated directly by ? or .map_err(Into::into) from a file outside allow is a finding at the call line, matched by name without the receiver type",
            "fan_out": "maximum number of other project directories whose source files one source file may use through resolved non-declaration references; default 10; crate roots listed in rust_roots and Python __main__.py are program entry points and exempt",
            "resolution": {
                "rust": "Rust 2018+ declared module trees, literal path attributes on file-level external modules, local crate mappings and module-level aliases; local and single external globs; Self and single inline trait bounds; observed prelude names with syntactic shadow checks; macro argument and macro_rules! body paths, literal and OUT_DIR include!, generated names of item-position macros, resource paths, standard/serde/external attributes and cfg over test and target platform predicates (all configurations measured); unsupported expansion and binding ambiguity remain incomplete",
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
                // DECISION: D156
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
        self.validate_disallowed()
    }
    fn validate_crates(&self) -> Result<()> {
        for (name, root) in &self.rust_crates {
            ensure!(
                identifier(name),
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
    // DECISION: D105
    fn validate_disallowed(&self) -> Result<()> {
        for entry in &self.disallowed {
            let literal = entry.path.split("::").all(identifier);
            ensure!(
                literal,
                "disallowed path must be a literal Rust path of :: separated identifiers: {}",
                entry.path
            );
            let explained = !entry.reason.trim().is_empty() && !entry.replacement.trim().is_empty();
            ensure!(
                explained,
                "disallowed path needs a nonempty reason and replacement: {}",
                entry.path
            );
            // DECISION: D125
            let named = entry.methods.iter().all(|method| identifier(method));
            ensure!(
                named,
                "disallowed methods must be nonempty Rust identifiers: {}",
                entry.path
            );
            entry.allowed()?;
        }
        Ok(())
    }
}

fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().enumerate().all(|(index, c)| {
            let later_digit = index > 0 && c.is_ascii_digit();
            c == '_' || c.is_ascii_alphabetic() || later_digit
        })
}
