// DECISION: D005
use super::{hook, project, reason};
use serde_json::{Value, json};
use std::path::Path;

const BYPASSES: [&str; 7] = [
    "git status",
    "just unknown",
    "just list && git status",
    r#"just run write echo "$(pwd)""#,
    "just --justfile other list",
    "just list\npwd",
    "just list write echo bypass",
];
const WRAPPERS: [&str; 2] = [
    "just run write -- sh -c 'git commit -m x'",
    "just read -- python3 tool.py",
];
const LISTINGS: [&str; 7] = [
    "just --list",
    "just list",
    "just lint-rule function-lines --example",
    "just lint-explain src/example.py --json",
    "just lint-edges src --json",
    "just run read -- git status",
    "just run write -- touch ok",
];

fn denied(root: &Path, command: &str) -> Value {
    let output = hook(root, "Bash", json!({"command": command}));
    let decision = &output["hookSpecificOutput"]["permissionDecision"];
    assert_eq!(decision, "deny", "{command}: {output}");
    output
}

// INVARIANT: I005
#[test]
fn shell_guard_rejects_bypass() {
    let (_temp, root) = project();
    for command in BYPASSES {
        denied(&root, command);
    }
}

#[test]
fn shell_guard_refuses_unreadable_check_entry() {
    let (_temp, root) = project();
    let output = denied(&root, r#"just scratch -- "git commit -q -m $'it\'s'""#);
    let reason = reason(&output);
    for text in ["git commit", "missing closing quote"] {
        assert!(reason.contains(text), "{reason}");
    }
}

#[test]
fn shell_base_programs_refuse_wrappers() {
    let (_temp, root) = project();
    for command in WRAPPERS {
        let output = denied(&root, command);
        let reason = reason(&output);
        for text in [" does not run ", "configured programs: "] {
            assert!(reason.contains(text), "{command}: {reason}");
        }
    }
}

#[test]
fn shell_catalog_and_recipe_listing() {
    let (_temp, root) = project();
    for command in LISTINGS {
        let output = hook(&root, "Bash", json!({"command": command}));
        assert!(!output.to_string().contains("deny"), "{command}: {output}");
    }
}
