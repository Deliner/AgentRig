// DECISION: D154
// DECISION: D239
// DECISION: D240
use super::{initialized, lint, text};
use crate::project_setup::init;
use crate::shared::testing::{invoke, repository, snapshot, write};
use anyhow::Result;
use serde_yaml_ng::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    thread,
};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n";
const TEAM: &str = "# team\nmax-struct-bools = 1\nmsrv = \"1.80\"\n\n[[disallowed-methods]]\npath = \"std::env::var\"\n";
const PREPENDED: &str = "avoid-breaking-exported-api = false\n";
const CONFLICT: &str = "max-struct-bools = 3\n";
const EFFECTIVE: [&str; 3] = ["Project/clippy.toml", "Project/.clippy.toml", "clippy.toml"];
const NARROWING: &str = "pub fn narrow(value: u64) -> u32 {
    let unused = 1;
    value as u32
}
";

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn yaml(path: &Path) -> Value {
    serde_yaml_ng::from_str(&read(path)).unwrap()
}

fn rust_project(manifest_directory: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    write(&root, &format!("{manifest_directory}/Cargo.toml"), MANIFEST);
    write(
        &root,
        &format!("{manifest_directory}/src/main.rs"),
        "fn main() {}\n",
    );
    (temp, root)
}

fn initialize(root: &Path, args: &[&str]) -> Result<i32> {
    let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
    init(root, &args)
}

fn clippy_files(root: &Path, directory: &Path) -> BTreeSet<PathBuf> {
    let mut found = BTreeSet::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let nested = path.is_dir();
        let profile = path.to_string_lossy().ends_with("clippy.toml");
        if nested {
            found.extend(clippy_files(root, &path));
        } else if profile {
            found.insert(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
    found
}

fn guided(found: &[serde_json::Value]) -> bool {
    let skill = |rule: &str| {
        let item = found.iter().find(|item| text(item, "rule") == rule);
        text(item.unwrap(), "skill").to_owned()
    };
    let guide = read(Path::new(&skill("clippy::as_conversions")));
    let named = guide.contains("`clippy::as_conversions`");
    named && skill("unused_variables").ends_with("/repair/SKILL.md")
}

#[test]
fn rust_scaffold_lints_its_crate_with_the_rig_clippy_profile() {
    let (_temp, root) = initialized("rust");
    let service = root.join(".agentrig");
    let module = yaml(&service.join("rust.yaml"));
    assert_eq!(module["manifest"], "Project/Cargo.toml", "{module:?}");
    let commands = yaml(&service.join("commands.yaml"));
    assert!(commands.get("clippy").is_none(), "{commands:?}");
    let checks = yaml(&service.join("checks.yaml"));
    let checks = checks.as_sequence().unwrap();
    assert!(checks.iter().all(|check| check["id"] != "clippy"));
    let thresholds = read(&repository().join("Project/clippy.toml"));
    assert_eq!(read(&root.join("clippy.toml")), thresholds);
    let library = root.join("Project/lib.rs");
    let clean = read(&library);
    fs::write(&library, format!("{NARROWING}{clean}")).unwrap();
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let place = |item: &serde_json::Value| {
        let rule = text(item, "rule");
        let path = text(item, "path");
        let line = &item["line"];
        format!("{rule} {path}:{line}")
    };
    let mut places: Vec<String> = found.iter().map(place).collect();
    places.sort();
    let expected = [
        "clippy::as_conversions Project/lib.rs:3",
        "unused_variables Project/lib.rs:2",
    ];
    assert_eq!(places, expected);
    assert!(guided(&found), "{found:?}");
    let check = invoke(&root, &["check", "--only", "lint"]);
    assert_eq!(check.code, Some(1), "{}", check.output());
    assert!(check.stderr.contains("ERROR [lint]"), "{}", check.output());
    fs::write(&library, clean).unwrap();
    assert_eq!(lint(&root), (Some(0), Vec::new()));
}

#[test]
fn rust_init_adds_missing_thresholds_to_the_effective_clippy_file() {
    let completes = |name: &str| {
        let (_temp, root) = rust_project("Project");
        write(&root, name, TEAM);
        let args = ["--language", "rust"];
        assert_eq!(initialize(&root, &args).unwrap(), 0, "{name}");
        assert_eq!(
            read(&root.join(name)),
            format!("{PREPENDED}{TEAM}"),
            "{name}"
        );
        let files = clippy_files(&root, &root);
        assert_eq!(files, BTreeSet::from([PathBuf::from(name)]), "{name}");
    };
    thread::scope(|scope| {
        let handles = EFFECTIVE.map(|name| scope.spawn(move || completes(name)));
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn rust_init_completes_the_file_clippy_reads_first() {
    let (_temp, root) = rust_project(".");
    write(&root, ".clippy.toml", TEAM);
    write(&root, "clippy.toml", CONFLICT);
    let args = ["--language", "rust"];
    assert_eq!(initialize(&root, &args).unwrap(), 0);
    assert_eq!(
        read(&root.join(".clippy.toml")),
        format!("{PREPENDED}{TEAM}")
    );
    assert_eq!(read(&root.join("clippy.toml")), CONFLICT);
}

#[test]
fn rust_init_refuses_a_conflicting_clippy_threshold() {
    let (_temp, root) = rust_project(".");
    write(&root, "clippy.toml", CONFLICT);
    let before = snapshot(&root);
    let args = ["--language", "rust"];
    let error = initialize(&root, &args).unwrap_err().to_string();
    assert!(error.contains("max-struct-bools = 3"), "{error}");
    assert_eq!(snapshot(&root), before);
}

#[test]
fn python_scaffold_has_no_clippy() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    assert_eq!(initialize(&root, &["--language", "python"]).unwrap(), 0);
    let commands = yaml(&root.join(".agentrig/commands.yaml"));
    assert!(commands.get("clippy").is_none(), "{commands:?}");
    let checks = yaml(&root.join(".agentrig/checks.yaml"));
    let checks = checks.as_sequence().unwrap();
    assert!(checks.iter().all(|check| check["id"] != "clippy"));
    assert_eq!(clippy_files(&root, &root), BTreeSet::new());
}
