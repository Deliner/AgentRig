// DECISION: D047
use super::{hook, project};
use crate::agent_hooks::dispatch;
use serde_json::{Value, json};
use std::{fs, path::Path};

const ARCHITECTURE: &str = "- include: ['Project/**']\n  skill: .agentrig/skills/refactor-large-directory/SKILL.md\n  events: [edit]\n  block: true\n";

fn route(root: &Path, route: &str) {
    let path = root.join(".agentrig/hooks.yaml");
    let hooks = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        hooks.replacen("routes:\n", &format!("routes:\n{route}"), 1),
    )
    .unwrap();
}

fn answer(value: &Value) -> (&str, &str) {
    let output = &value["hookSpecificOutput"];
    let decision = output["permissionDecision"].as_str().unwrap_or("allow");
    let reason = output["permissionDecisionReason"].as_str();
    (
        decision,
        reason
            .or(output["additionalContext"].as_str())
            .unwrap_or(""),
    )
}

fn edit(root: &Path, path: &str) -> Value {
    hook(root, "Edit", json!({"file_path": path}))
}

#[test]
fn first_edit_in_an_area_stops_until_its_requirement_is_delivered() {
    let (_temp, root) = project();
    route(&root, ARCHITECTURE);
    let first = edit(&root, "Project/agent_hooks/mod.rs");
    let (decision, message) = answer(&first);
    assert_eq!(decision, "deny");
    for expected in [
        "stopped this Edit call before it ran",
        "- Required: read and apply",
        "refactor-large-directory/SKILL.md for edit of Project/agent_hooks/mod.rs.",
        "Source: hooks.routes[0].",
    ] {
        assert!(message.contains(expected), "{expected}: {message}");
    }
    assert_eq!(edit(&root, "Project/agent_hooks/mod.rs"), Value::Null);
    assert_eq!(edit(&root, "Project/agent_hooks/guard.rs"), Value::Null);
    assert_eq!(
        answer(&edit(&root, "Project/agent_hooks/tests/mod.rs")).0,
        "deny"
    );
    let compact = json!({"hook_event_name": "SessionStart", "source": "compact"});
    dispatch(&root, &compact).unwrap();
    assert_eq!(answer(&edit(&root, "Project/agent_hooks/mod.rs")).0, "deny");
}

fn patched(files: &[&str]) -> String {
    let (_temp, root) = project();
    route(&root, ARCHITECTURE);
    fs::write(
        root.join(".agentrig/reminder.json"),
        r#"{"attention_interval_tokens": 100, "full_refresh_interval_tokens": 300}"#,
    )
    .unwrap();
    let transcript = root.join("transcript.jsonl");
    fs::write(&transcript, "").unwrap();
    let common = json!({"session_id": "s", "transcript_path": transcript, "cwd": root});
    let mut start = common.clone();
    start["hook_event_name"] = json!("SessionStart");
    dispatch(&root, &start).unwrap();
    let usage = json!({"type": "event_msg", "payload": {"type": "token_count",
        "info": {"last_token_usage": {"input_tokens": 400}}}});
    fs::write(&transcript, format!("{usage}\n")).unwrap();
    let patch: String = files
        .iter()
        .map(|file| format!("*** Update File: {file}\n"))
        .collect();
    let mut event = common;
    event["hook_event_name"] = json!("PreToolUse");
    event["tool_name"] = json!("apply_patch");
    event["tool_input"] = json!({"command": format!("*** Begin Patch\n{patch}*** End Patch")});
    let output = dispatch(&root, &event).unwrap().unwrap();
    let (decision, message) = answer(&output);
    assert_eq!(decision, "deny");
    message.replace(&root.display().to_string(), "ROOT")
}

#[test]
fn one_response_keeps_every_requirement_in_configured_order() {
    let files = [
        "Project/b/x.rs",
        "Ledger/Plan.md",
        "Project/a/y.rs",
        "Ledger/Archive/Plan.md",
        "Project/a/z.rs",
    ];
    let message = patched(&files);
    let mut reversed = files;
    reversed.reverse();
    assert_eq!(patched(&reversed), message);
    let architecture = "- Required: read and apply ROOT/.agentrig/skills/refactor-large-directory/SKILL.md for edit of Project/a/y.rs; edit of Project/a/z.rs; edit of Project/b/x.rs. Source: hooks.routes[0].";
    let plan = "- Guidance: read and apply ROOT/.agentrig/skills/edit-plan/SKILL.md for edit of Ledger/Plan.md; edit of Ledger/Archive/Plan.md. Source: hooks.routes[1], hooks.routes[2].";
    let refresh = message.find("COMPLEXITY_DISCIPLINE_FULL_REFRESH_REQUIRED");
    let positions = [refresh, message.find(architecture), message.find(plan)];
    assert!(positions.iter().all(Option::is_some), "{message}");
    assert!(positions.is_sorted(), "{message}");
}

#[test]
fn contradictory_route_order_is_rejected_until_repaired() {
    let (_temp, root) = project();
    let hooks = root.join(".agentrig/hooks.yaml");
    let valid = fs::read_to_string(&hooks).unwrap();
    route(
        &root,
        "- include: [Ledger/State.md]\n  skill: .agentrig/skills/edit-state/SKILL.md\n",
    );
    let denied = edit(&root, "Project/main.py");
    let (decision, message) = answer(&denied);
    assert_eq!(decision, "deny");
    let conflict = ".agentrig/hooks.yaml: hooks.routes[0] and hooks.routes[6] order .agentrig/skills/edit-state/SKILL.md both before and after .agentrig/skills/edit-plan/SKILL.md (hooks.routes[1])";
    assert!(message.contains(conflict), "{message}");
    assert_eq!(edit(&root, ".agentrig/hooks.yaml"), Value::Null);
    fs::write(&hooks, valid).unwrap();
    assert_eq!(edit(&root, "Project/main.py"), Value::Null);
}

#[test]
fn read_only_catalog_commands_carry_no_shell_guidance() {
    let (_temp, root) = project();
    let read = json!({"command": "just run read -- git status"});
    assert_eq!(hook(&root, "Bash", read), Value::Null);
    let write = hook(
        &root,
        "Bash",
        json!({"command": "just run write -- touch x"}),
    );
    let (decision, message) = answer(&write);
    assert_eq!(decision, "allow");
    assert!(message.contains("for opaque shell write"), "{message}");
}

#[test]
fn vac_check_entry_reminds_state_once_per_context() {
    let (_temp, root) = project();
    let commit = "just run write -- git commit -q -m done";
    let first = hook(&root, "exec_command", json!({"cmd": commit}));
    let (decision, message) = answer(&first);
    assert_eq!(decision, "deny");
    let state = "- Required: read and apply {root}/.agentrig/skills/edit-state/SKILL.md for opaque shell write; VAC check entry. Source: hooks.routes[5], hooks.routes[6].";
    let state = state.replace("{root}", &root.display().to_string());
    assert!(message.contains(&state), "{message}");
    assert!(message.contains("- Guidance: read and apply"), "{message}");
    assert_eq!(hook(&root, "Bash", json!({"command": commit})), Value::Null);
    assert_eq!(
        hook(&root, "Bash", json!({"command": "just check"})),
        Value::Null
    );
    let write = hook(
        &root,
        "Bash",
        json!({"command": "just run write -- touch ok"}),
    );
    assert_eq!(write, Value::Null);
}
