use super::Context;
use crate::shared::testing::{FAIL, GATE, REPAIR, invoke, project};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const STRICT: [&str; 3] = [
    "version: \"1\"",
    "version: 1\nversion: 1",
    "version: &schema 1",
];

fn configured(addition: &str, checks: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, checks);
    let identity = root.join(".agentrig/project.yaml");
    let text = fs::read_to_string(&identity).unwrap();
    fs::write(&identity, text + addition).unwrap();
    (temp, root)
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
    let rejected = invoke(&root, &["config-check"]);
    assert_eq!(rejected.code, Some(2), "{}", rejected.output());
    let named = rejected.stderr.contains("lint check is configured");
    assert!(named, "{}", rejected.stderr);
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
        let identity = root.join(".agentrig/project.yaml");
        let text = fs::read_to_string(&identity).unwrap();
        fs::write(&identity, text.replacen("version: 1", replacement, 1)).unwrap();
        let result = invoke(&root, &["config-check"]);
        assert_eq!(result.code, Some(2), "{replacement}: {}", result.output());
        let (message, action) = result.stderr.split_once("ACTION:").unwrap();
        let named = message.contains(".agentrig/project.yaml") && !action.is_empty();
        assert!(named, "{replacement}: {}", result.stderr);
    }
}
