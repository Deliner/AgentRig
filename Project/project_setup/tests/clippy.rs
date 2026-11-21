// DECISION: D132
// DECISION: D154
use super::{initialized, lint, text};
use crate::project_setup::init;
use crate::shared::strict_yaml;
use crate::shared::testing::{repository, snapshot, write};
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

fn strings(value: &Value) -> Vec<&str> {
    let items = value.as_sequence().unwrap().iter();
    items.map(|item| item.as_str().unwrap()).collect()
}

fn denied(argv: &[&str]) -> BTreeSet<String> {
    let start = argv.iter().position(|arg| *arg == "--").unwrap() + 1;
    let flags = &argv[start..];
    let switches: BTreeSet<&str> = flags.iter().step_by(2).copied().collect();
    assert_eq!(switches, BTreeSet::from(["-D"]), "{flags:?}");
    flags
        .iter()
        .skip(1)
        .step_by(2)
        .map(|lint| (*lint).to_owned())
        .collect()
}

#[test]
fn rust_scaffold_delivers_the_dev_gate_clippy_profile() {
    let (_temp, root) = rust_project(".");
    let args = ["--language", "rust"];
    assert_eq!(initialize(&root, &args).unwrap(), 0);
    let commands = yaml(&root.join(".agentrig/commands.yaml"));
    let argv = strings(&commands["clippy"]["argv"]);
    assert_eq!(
        argv[..5],
        ["cargo", "clippy", "--manifest-path", "Cargo.toml", "--"]
    );
    let gate = yaml(&repository().join(".agentrig/commands.yaml"));
    assert_eq!(denied(&argv), denied(&strings(&gate["clippy"]["argv"])));
    let checks = yaml(&root.join(".agentrig/checks.yaml"));
    let check = checks
        .as_sequence()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "clippy");
    let check = check.unwrap();
    assert_eq!(
        (&check["kind"], &check["command"]),
        (&"command".into(), &"clippy".into())
    );
    let thresholds = read(&repository().join("Project/clippy.toml"));
    assert_eq!(read(&root.join("clippy.toml")), thresholds);
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

// DECISION: D184
#[test]
fn shipped_rules_check_lowering_flags_of_the_clippy_command() {
    let (_temp, root) = initialized("rust");
    let commands = root.join(".agentrig/commands.yaml");
    let (_, before) = lint(&root);
    let mut config: serde_json::Value = strict_yaml::read(&commands).unwrap();
    let argv = config["clippy"]["argv"].as_array_mut().unwrap();
    argv.extend(["--cap-lints", "allow"].map(serde_json::Value::from));
    fs::write(&commands, strict_yaml::encode(&config).unwrap()).unwrap();
    let (code, after) = lint(&root);
    let skills = |found: &[serde_json::Value]| -> Vec<String> {
        let reported = |item: &&serde_json::Value| {
            text(item, "rule") == "allow-justification"
                && text(item, "path") == ".agentrig/commands.yaml"
        };
        let skill = |item: &serde_json::Value| text(item, "skill").to_owned();
        found.iter().filter(reported).map(skill).collect()
    };
    assert_eq!(skills(&before), Vec::<String>::new(), "{before:?}");
    assert_eq!(code, Some(1), "{after:?}");
    let after = skills(&after);
    assert_eq!(after.len(), 1, "{after:?}");
    assert!(after[0].ends_with("justify-allow/SKILL.md"), "{after:?}");
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
