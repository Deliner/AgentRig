// DECISION: D013
// DECISION: D014
// DECISION: D015
use super::{initialized, pre_tool, project, respond};
use crate::shared::testing::{run, runtime};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::symlink, path::Path};

const LEDGER_EDITS: [(&str, &str, &[&str]); 19] = [
    ("Edit", r#"{"file_path": "Ledger/Plan.md"}"#, &["edit-plan"]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Backlog.md"}"#,
        &["edit-backlog"],
    ),
    (
        "Write",
        r#"{"file_path": "Ledger/Backlog/001.md"}"#,
        &["edit-backlog"],
    ),
    ("Write", r#"{"path": "Ledger/State.md"}"#, &["edit-state"]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Decisions.md"}"#,
        &["edit-decisions"],
    ),
    (
        "Write",
        r#"{"file_path": "Ledger/Invariants.md"}"#,
        &["edit-invariants"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Plan/001.md"}"#,
        &["edit-plan"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Decisions/001.md"}"#,
        &["edit-decisions"],
    ),
    (
        "Edit",
        r#"{"file_path": "Ledger/Invariants/001.md"}"#,
        &["edit-invariants"],
    ),
    ("Edit", r#"{"file_path": "docs/State.md"}"#, &[]),
    ("Edit", r#"{"file_path": "Ledger/Plan-other.md"}"#, &[]),
    (
        "Edit",
        r#"{"file_path": "Ledger/Plan/../State.md"}"#,
        &["edit-state"],
    ),
    (
        "Edit",
        r#"{"file_path": "{root}/Ledger/State.md"}"#,
        &["edit-state"],
    ),
    ("Edit", r#"{"file_path": "{outside}/Ledger/State.md"}"#, &[]),
    (
        "apply_patch",
        r#""*** Begin Patch\n*** Update File: Ledger/Plan.md\n*** Update File: Ledger/Plan/001.md\n*** Delete File: Ledger/State.md\n*** Add File: Ledger/Invariants/013.md\n*** End Patch""#,
        &["edit-plan", "edit-invariants", "edit-state"],
    ),
    (
        "apply_patch",
        r#"{"patch": "*** Update File: Ledger/Decisions/001.md\n*** Move to: Ledger/Invariants/001.md\n"}"#,
        &["edit-decisions", "edit-invariants"],
    ),
    (
        "apply_patch",
        r#"{"input": "*** Add File: Ledger/State.md\n+# State\n"}"#,
        &["edit-state"],
    ),
    (
        "apply_patch",
        r#"{"command": "*** Begin Patch\n*** Update File: Ledger/Plan.md\n*** End Patch"}"#,
        &["edit-plan"],
    ),
    (
        "apply_patch",
        r#"{"patch_text": "*** Update File: docs/example.md\n+*** Update File: Ledger/Plan.md\n"}"#,
        &[],
    ),
];
const MEMORY_SKILLS: [&str; 5] = [
    "edit-plan",
    "edit-backlog",
    "edit-decisions",
    "edit-invariants",
    "edit-state",
];

fn in_session(root: &Path, mut event: Value, session: &str) -> Value {
    event["session_id"] = json!(session);
    respond(root, &event)
}

fn guidance(root: &Path, output: &Value, expected: &[&str]) {
    let unguided = expected.is_empty();
    if unguided {
        assert!(output.is_null(), "{output}");
        return;
    }
    let specific = &output["hookSpecificOutput"];
    assert!(specific.get("permissionDecision").is_none(), "{output}");
    let message = specific["additionalContext"].as_str().unwrap();
    for skill in MEMORY_SKILLS {
        let path = root.join("guides").join(skill).join("SKILL.md");
        assert!(path.is_file(), "{}", path.display());
        let count = message.matches(path.to_str().unwrap()).count();
        let delivered = usize::from(expected.contains(&skill));
        assert_eq!(count, delivered, "{skill}: {message}");
    }
}

// INVARIANT: I012
#[test]
fn ledger_edit_guidance() {
    let (_temp, root) = initialized(&["--memory", "Ledger", "--skills", "guides"]);
    let outside = root.parent().unwrap().to_str().unwrap();
    for (index, (tool, payload, expected)) in LEDGER_EDITS.into_iter().enumerate() {
        let payload = payload.replace("{root}", root.to_str().unwrap());
        let payload = serde_json::from_str(&payload.replace("{outside}", outside)).unwrap();
        let output = in_session(
            &root,
            pre_tool(&root, tool, payload),
            &format!("edit-{index}"),
        );
        guidance(&root, &output, expected);
    }
}

#[test]
fn nested_cwd_and_symlinks() {
    let (_temp, root) = initialized(&["--memory", "notes"]);
    let event = pre_tool(&root.join("notes"), "Edit", json!({"file_path": "Plan.md"}));
    let guided = in_session(&root, event.clone(), "direct");
    assert!(guided.to_string().contains("edit-plan"), "{guided}");
    let path = root.join("notes/Plan.md");
    fs::remove_file(&path).unwrap();
    symlink(root.parent().unwrap().join("outside.md"), &path).unwrap();
    assert!(in_session(&root, event, "linked").is_null());
}

#[test]
fn storage_failure_keeps_recovery_context() {
    let (_temp, root) = project();
    let storage = root.join(".agentrig/runtime");
    fs::remove_dir(&storage).unwrap();
    fs::write(&storage, "occupied").unwrap();
    let session = r#"{"hook_event_name": "SessionStart"}"#;
    let result = run(&mut runtime(&root, &["hook"]), session);
    let reminders = storage.join("reminders");
    let recovery = [
        "Ledger/State.md",
        "FULL_REFRESH_REQUIRED",
        "Hook session state unavailable",
        reminders.to_str().unwrap(),
    ];
    for text in recovery {
        assert!(result.stdout.contains(text), "{text}: {}", result.stdout);
    }
    let stderr = &result.stderr;
    assert!(stderr.contains("session reminder state"), "{stderr}");
}
