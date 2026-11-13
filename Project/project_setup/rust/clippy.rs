use super::cargo::Layout;
use crate::shared::fs_with_context as fs;
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

// DECISION: D154
const SETTINGS: &str = include_str!("../../clippy.toml");
const FILES: [&str; 2] = [".clippy.toml", "clippy.toml"];

pub struct Settings {
    path: PathBuf,
    contents: String,
}

impl Settings {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn write(&self) -> Result<()> {
        Ok(fs::write(&self.path, &self.contents)?)
    }
}

// DECISION: D154
pub fn settings(root: &Path, rust: &Layout) -> Result<Settings> {
    let manifest = root.join(&rust.manifest);
    let directory = manifest
        .parent()
        .context("Cargo manifest has no directory")?;
    let ancestors = directory
        .ancestors()
        .take_while(|ancestor| ancestor.starts_with(root));
    let (path, existing) = match effective(ancestors) {
        Some(path) => {
            let existing = fs::read_to_string(&path)?;
            (path, existing)
        }
        None => (root.join("clippy.toml"), String::new()),
    };
    let present: toml::Table =
        toml::from_str(&existing).with_context(|| format!("parse {}", path.display()))?;
    let required: toml::Table = toml::from_str(SETTINGS)?;
    let mut missing = String::new();
    for (key, value) in required {
        match present.get(&key) {
            Some(found) => ensure!(
                *found == value,
                "init collision: {} sets {key} = {found}, AgentRig requires {value}; existing files were preserved",
                path.display()
            ),
            None => missing.push_str(&format!("{key} = {value}\n")),
        }
    }
    Ok(Settings {
        contents: missing + &existing,
        path,
    })
}

fn effective<'a>(directories: impl Iterator<Item = &'a Path>) -> Option<PathBuf> {
    directories
        .flat_map(|ancestor| FILES.map(|name| ancestor.join(name)))
        .find(|path| path.is_file())
}

// DECISION: D132
// DECISION: D137
// DECISION: D152
// DECISION: D153
const PROFILE: &[&str] = &[
    "panic",
    "todo",
    "unimplemented",
    "dbg_macro",
    "unreachable",
    "expect_used",
    "unwrap_used",
    "let_underscore_must_use",
    "exit",
    "should_panic_without_expect",
    "missing_fields_in_debug",
    "rest_pat_in_fully_bound_structs",
    "self_named_module_files",
    "print_stdout",
    "print_stderr",
    "string_slice",
    "iter_over_hash_type",
    "collection_is_never_read",
    "read_zero_byte_vec",
    "unchecked_time_subtraction",
    "map_err_ignore",
    "partial_pub_fields",
    "struct_excessive_bools",
    "option_option",
    "useless_let_if_seq",
    "return_self_not_must_use",
    "multiple_unsafe_ops_per_block",
    "significant_drop_in_scrutinee",
    "uninlined_format_args",
    "wildcard_enum_match_arm",
    "as_conversions",
    "allow_attributes",
];

pub fn argv(rust: &Layout) -> Vec<String> {
    let mut argv: Vec<String> = ["cargo", "clippy", "--manifest-path"]
        .into_iter()
        .map(String::from)
        .collect();
    argv.push(rust.manifest.clone());
    argv.extend(["--", "-D", "warnings"].map(String::from));
    for lint in PROFILE {
        argv.push("-D".into());
        argv.push(format!("clippy::{lint}"));
    }
    argv
}
