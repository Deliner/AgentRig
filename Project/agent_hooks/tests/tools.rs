// DECISION: D200
use super::{hook, project, reason};
use serde_json::{Value, json};
use std::fs;

fn refused(output: &Value, tool: &str) {
    let decision = &output["hookSpecificOutput"]["permissionDecision"];
    assert_eq!(decision, "deny", "{tool}: {output}");
    let named = reason(output).contains(&format!("this {tool} call"));
    assert!(named, "{tool}: {output}");
}

#[test]
fn monitor_commands_pass_the_same_shell_checks_and_websocket_watches_pass() {
    let (_temp, root) = project();
    for command in ["echo bypass", "just list"] {
        let monitor = hook(&root, "Monitor", json!({"command": command}));
        let shell = hook(&root, "Bash", json!({"command": command}));
        assert_eq!(reason(&monitor), reason(&shell), "{command}");
    }
    let bypass = hook(&root, "Monitor", json!({"command": "echo bypass"}));
    assert!(!reason(&bypass).is_empty(), "{bypass}");
    let socket = json!({"ws": {"url": "wss://example.test"}});
    let watch = hook(&root, "Monitor", socket);
    assert!(watch.is_null(), "{watch}");
}

#[test]
fn notebook_edits_get_the_routes_of_the_file_they_change() {
    let (_temp, root) = project();
    let notebook = root.join("Ledger/Plan.md");
    let output = hook(&root, "NotebookEdit", json!({"notebook_path": notebook}));
    assert!(output.to_string().contains("edit-plan"), "{output}");
}

#[test]
fn tools_the_hook_cannot_check_are_refused_by_name() {
    let (_temp, root) = project();
    let tools = [
        ("mcp__example__write_file", json!({"path": "README.md"})),
        ("Skill", json!({"skill": "edit-plan"})),
        ("EnterWorktree", json!({})),
        ("Agent", json!({"prompt": "work", "isolation": "worktree"})),
        ("Monitor", json!({"description": "nothing to watch"})),
    ];
    for (tool, input) in tools {
        refused(&hook(&root, tool, input), tool);
    }
}

#[test]
fn tools_that_neither_run_commands_nor_write_files_pass() {
    let (_temp, root) = project();
    let tools = [
        ("Read", json!({"file_path": "Ledger/State.md"})),
        ("Agent", json!({"prompt": "work"})),
        ("update_plan", json!({"plan": []})),
    ];
    for (tool, input) in tools {
        let output = hook(&root, tool, input);
        assert!(output.is_null(), "{tool}: {output}");
    }
}

#[test]
fn invalid_configuration_keeps_refusing_unknown_tools_and_passing_passive_ones() {
    let (_temp, root) = project();
    fs::write(root.join(".agentrig/commands.yaml"), "read:\n  argv: []\n").unwrap();
    let tool = "mcp__example__write_file";
    refused(&hook(&root, tool, json!({"path": "README.md"})), tool);
    let read = hook(&root, "Read", json!({"file_path": "README.md"}));
    assert!(read.is_null(), "{read}");
}
