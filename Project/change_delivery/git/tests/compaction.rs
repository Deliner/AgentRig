// DECISION: D230
use super::{commit, installed, start};
use crate::shared::testing::{invoke, run, runtime};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

const RUNTIME: &str = ".cache/runtime";
const COMPACT: &str = "mcp__self-compact__compact";
const UNLIFTED: &str = "could not lift the compaction requirement";

fn hook(root: &Path, event: &Value) -> Value {
    let answered = run(
        &mut runtime(root, &["hook", "--agent", "claude-code"]),
        &event.to_string(),
    );
    assert_eq!(answered.code, Some(0), "{}", answered.output());
    serde_json::from_str(&answered.stdout).unwrap_or(Value::Null)
}

fn refusal(root: &Path, tool: &str) -> String {
    let event = json!({"hook_event_name": "PreToolUse", "tool_name": tool,
        "tool_input": {"file_path": "README.md"}, "cwd": root});
    let answer = hook(root, &event);
    let reason = &answer["hookSpecificOutput"]["permissionDecisionReason"];
    reason.as_str().unwrap_or_default().to_owned()
}

fn session(root: &Path, source: &str) -> String {
    let event = json!({"hook_event_name": "SessionStart", "source": source, "cwd": root});
    let answer = hook(root, &event);
    let context = &answer["hookSpecificOutput"]["additionalContext"];
    context.as_str().unwrap_or_default().to_owned()
}

fn merged(root: &Path, name: &str) -> String {
    start(root, name);
    commit(root, &format!("{name}.txt"), name);
    let merged = invoke(root, &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    merged.stdout.lines().last().unwrap_or_default().to_owned()
}

#[test]
fn a_merge_refuses_every_tool_but_compaction_until_the_context_is_renewed() {
    let (_temp, root) = installed();
    for (index, renewal) in ["compact", "clear", "startup"].into_iter().enumerate() {
        let instruction = merged(&root, &format!("product-{index}"));
        assert!(instruction.contains("/compact"), "{instruction}");
        assert!(instruction.contains(COMPACT), "{instruction}");
        for tool in ["Read", "Write", "Bash", "mcp__other__tool"] {
            assert_eq!(refusal(&root, tool), instruction, "{tool}");
        }
        assert_eq!(refusal(&root, COMPACT), "");
        for kept in ["resume", "fork"] {
            session(&root, kept);
            assert_eq!(refusal(&root, "Read"), instruction, "{kept}");
        }
        let context = session(&root, renewal);
        assert!(!context.contains(UNLIFTED), "{context}");
        assert_eq!(refusal(&root, "Read"), "", "{renewal}");
    }
}

#[test]
fn branch_commits_and_a_failed_merge_require_no_compaction() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "product.txt", "product");
    assert_eq!(refusal(&root, "Read"), "");
    let blocked = run(
        runtime(&root, &["feature-merge"]).env("BLOCK_DELIVERY", "1"),
        "",
    );
    assert_eq!(blocked.code, Some(1), "{}", blocked.output());
    assert!(!blocked.stdout.contains("/compact"), "{}", blocked.stdout);
    assert_eq!(refusal(&root, "Read"), "");
}

#[test]
fn an_unreadable_requirement_refuses_tools_with_its_cause() {
    let (_temp, root) = installed();
    let runtime_directory = root.join(RUNTIME);
    let sealed = fs::Permissions::from_mode(0o000);
    fs::set_permissions(&runtime_directory, sealed).unwrap();
    let refused = refusal(&root, "Read");
    let failed = session(&root, "startup");
    fs::set_permissions(&runtime_directory, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        refused.contains("could not read whether a merge requires"),
        "{refused}"
    );
    assert!(refused.contains("Permission denied"), "{refused}");
    assert!(failed.contains(UNLIFTED), "{failed}");
    assert_eq!(refusal(&root, COMPACT), "");
}
