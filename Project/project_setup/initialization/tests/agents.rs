// DECISION: D188
use crate::project_setup::initialization::init;
use crate::shared::calling_agent::Agent;
use crate::shared::testing::{hermetic, invoke, run, snapshot};
use serde_json::{Value, json};
use std::{fs, path::Path};

const CLIENT_FILES: [&str; 4] = [
    ".codex/config.toml",
    ".codex/hooks.json",
    ".claude/settings.json",
    ".claude/CLAUDE.md",
];

fn registration(root: &Path, agent: Agent) -> Value {
    let file = match agent {
        Agent::Codex => ".codex/hooks.json",
        Agent::ClaudeCode => ".claude/settings.json",
    };
    serde_json::from_str(&fs::read_to_string(root.join(file)).unwrap()).unwrap()
}

fn hook(root: &Path, handler: &Value, event: &Value) -> Value {
    let mut hook = hermetic("sh");
    let command = handler["command"].as_str().unwrap();
    hook.args(["-c", command]).current_dir(root);
    let result = run(&mut hook, &event.to_string());
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

#[test]
fn init_registers_both_agents_and_each_sees_skills_through_its_directory() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let note = fs::read_to_string(root.join(".claude/CLAUDE.md")).unwrap();
    assert_eq!(note, "@../AGENTS.md\n");
    assert!(!root.join("CLAUDE.md").exists());
    assert!(!root.join(".mcp.json").exists());
    for agent in Agent::ALL {
        let settings = registration(&root, agent);
        let handler = &settings["hooks"]["PreToolUse"][0]["hooks"][0];
        assert_eq!(handler["timeout"], 30, "{agent:?}");
        let flag = format!("--agent {}", agent.name());
        assert!(
            handler["command"].as_str().unwrap().ends_with(&flag),
            "{handler}"
        );
        let shell = json!({"hook_event_name": "PreToolUse", "tool_name": "Bash",
            "tool_input": {"command": "echo bypass"}, "cwd": root, "session_id": agent.name()});
        let denied = hook(&root, handler, &shell);
        let decision = &denied["hookSpecificOutput"]["permissionDecision"];
        assert_eq!(decision, "deny", "{agent:?}: {denied}");
        let state = json!({"hook_event_name": "PreToolUse", "tool_name": "Write",
            "tool_input": {"file_path": "Ledger/State.md"}, "cwd": root, "session_id": agent.name()});
        let guided = hook(&root, handler, &state).to_string();
        let skill = root.join(agent.skills()).join("edit-state/SKILL.md");
        assert!(
            guided.contains(skill.to_str().unwrap()),
            "{agent:?}: {guided}"
        );
    }
}

#[test]
fn init_collision_with_an_agent_file_preserves_files() {
    for existing in CLIENT_FILES {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join(existing);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{\"model\": \"consumer-model\"}\n").unwrap();
        let before = snapshot(&root);
        let error = init(&root, &[]).unwrap_err().to_string();
        assert!(error.contains("collision"), "{existing}: {error}");
        assert_eq!(snapshot(&root), before, "{existing}");
    }
}
