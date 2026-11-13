use crate::project_config::{Context, VERSION};
use crate::shared::testing::{FAIL, GATE, REPAIR, Run, invoke, project};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const READ: &str =
    "read:\n  argv: []\n  accepts_args: true\n  programs: {cat: {}, sh: {}}\n  read_only: true\n";
const INVALID: [(&str, &str, &str); 13] = [
    ("project.yaml", VERSION, "9.0.0"),
    ("project.yaml", "version: 1", "version: 7"),
    ("project.yaml", "paths:", "frontend: unknown\npaths:"),
    ("project.yaml", "memory: notes", "memory: ../outside"),
    ("commands.yaml", "read_only: true", "read_only: \"true\""),
    ("commands.yaml", "accepts_args: true", "accept_arg: true"),
    ("project.yaml", "skills: guides", "skills: /outside"),
    ("git.yaml", "base: trunk", "base: bad branch"),
    ("git.yaml", "base: trunk", "base: topic.lock"),
    ("git.yaml", "prefix: task/", "prefix: trunk"),
    ("git.yaml", "prefix: task/", "prefix: task//"),
    (
        "git.yaml",
        "prefix: task/",
        "prefix: task/\nremote: --mirror",
    ),
    ("project.yaml", "paths:", "lifetime: forever\npaths:"),
];
const STRICT: [&str; 3] = [
    "version: \"1\"",
    "version: 1\nversion: 1",
    "version: &schema 1",
];

fn configured(addition: &str, checks: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, &format!("{FAIL}{READ}"), checks);
    let identity = root.join(".agentrig/project.yaml");
    let text = fs::read_to_string(&identity).unwrap();
    fs::write(&identity, text + addition).unwrap();
    (temp, root)
}

fn change(root: &Path, module: &str, old: &str, new: &str) {
    let path = root.join(".agentrig").join(module);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains(old), "{module}: {text}");
    fs::write(&path, text.replace(old, new)).unwrap();
}

fn rejected(root: &Path, case: &str) -> Run {
    let result = invoke(root, &["config-check"]);
    assert_eq!(result.code, Some(2), "{case}: {}", result.output());
    assert!(
        result.stderr.contains("ACTION:"),
        "{case}: {}",
        result.stderr
    );
    result
}

#[test]
fn disabled_lint_requires_consistent_gate() {
    let (_temp, root) = configured("capabilities:\n  lint: false\n", "[]\n");
    fs::remove_file(root.join("lint.yaml")).unwrap();
    let checked = invoke(&root, &["config-check"]);
    assert_eq!(checked.code, Some(0), "{}", checked.output());
    let linted = invoke(&root, &["lint"]);
    assert_eq!(linted.code, Some(2), "{}", linted.output());
    let refused = linted.stderr.contains("capabilities.lint is disabled");
    assert!(refused, "{}", linted.stderr);
    let check = format!("- id: lint\n  kind: lint\n  skill: {REPAIR}\n");
    fs::write(root.join(".agentrig/checks.yaml"), check).unwrap();
    let configured = rejected(&root, "lint check");
    let named = configured.stderr.contains("lint check is configured");
    assert!(named, "{}", configured.stderr);
}

#[test]
fn unknown_capability_is_named() {
    for capability in ["unknown", "review", "delegation"] {
        let addition = format!("capabilities:\n  {capability}: true\n");
        let (_temp, root) = configured(&addition, GATE);
        let error = format!("{:#}", Context::load(&root).err().unwrap());
        let named = error.contains(&format!("unknown field `{capability}`"));
        assert!(named, "{capability}: {error}");
    }
}

#[test]
fn root_yaml_is_strict() {
    for replacement in STRICT {
        let (_temp, root) = configured("", GATE);
        change(&root, "project.yaml", "version: 1", replacement);
        let result = rejected(&root, replacement);
        let (message, _) = result.stderr.split_once("ACTION:").unwrap();
        let named = message.contains(".agentrig/project.yaml");
        assert!(named, "{replacement}: {}", result.stderr);
    }
}

#[test]
fn configuration_errors_are_actionable() {
    let (_valid, root) = configured("", GATE);
    let accepted = invoke(&root, &["config-check"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    for (module, old, new) in INVALID {
        let (_temp, root) = configured("", GATE);
        change(&root, module, old, new);
        rejected(&root, new);
    }
}

// DECISION: D038
#[test]
fn project_module_rejects_another_module_section() {
    let (_temp, root) = configured("commands: {}\n", GATE);
    let result = rejected(&root, "commands");
    let moved = result
        .stderr
        .contains("move commands to .agentrig/commands.yaml");
    assert!(moved, "{}", result.stderr);
}
