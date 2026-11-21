use crate::project_setup::init;
use crate::shared::testing::invoke;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

mod clippy;
mod coverage;
mod defaults;
mod delivery;
mod doctor;
mod recipes;
mod runtime;
mod rustflags;
mod toolchain;

fn installed_runtime(root: &Path) -> PathBuf {
    let mut runtimes = fs::read_dir(root.join(".agentrig/bin"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("agentrig"))
        .filter(|path| path.is_file());
    let runtime = runtimes.next().unwrap();
    assert_eq!(runtimes.next(), None);
    runtime
}

fn initialized(language: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let args = ["--language", language].map(str::to_owned);
    assert_eq!(init(&root, &args).unwrap(), 0);
    (temp, root)
}

fn lint(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = invoke(root, &["lint", "--json"]);
    let found = serde_json::from_str(&result.stdout);
    let found = found.unwrap_or_else(|error| panic!("{error}: {}", result.output()));
    (result.code, found)
}

fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap()
}
