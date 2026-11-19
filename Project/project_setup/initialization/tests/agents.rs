// DECISION: D188
use crate::project_setup::initialization::init;
use crate::shared::calling_agent::Agent;
use crate::shared::testing::{hermetic, invoke, run, runtime, snapshot};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
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
        let every_tool = &settings["hooks"]["PreToolUse"][0];
        assert_eq!(every_tool["matcher"], "*", "{agent:?}");
        let handler = &every_tool["hooks"][0];
        assert_eq!(handler["timeout"], 30, "{agent:?}");
        let flag = format!("--agent {} ", agent.name());
        assert!(
            handler["command"].as_str().unwrap().contains(&flag),
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

// DECISION: D200
#[test]
fn a_hook_that_cannot_start_blocks_the_call_with_its_reason() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let launcher = root.join(".agentrig/bin/agentrig");
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o644)).unwrap();
    blocked(&root, "not executable");
    fs::remove_file(&launcher).unwrap();
    blocked(&root, "missing");
}

fn blocked(root: &Path, breakage: &str) {
    let event = json!({"hook_event_name": "PreToolUse", "tool_name": "Read",
        "tool_input": {"file_path": "README.md"}, "cwd": root});
    for agent in Agent::ALL {
        let settings = registration(root, agent);
        let handler = &settings["hooks"]["PreToolUse"][0]["hooks"][0];
        let mut hook = hermetic("sh");
        hook.args(["-c", handler["command"].as_str().unwrap()]);
        let result = run(hook.current_dir(root), &event.to_string());
        assert_eq!(result.code, Some(2), "{breakage} {agent:?}");
        let reason = result.stderr.contains("AgentRig hook could not run");
        assert!(reason, "{breakage} {agent:?}: {}", result.output());
    }
}

#[test]
fn lint_names_the_skill_through_the_directory_of_the_agent_its_variable_names() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let wide = format!("# Title\n\n{}\n", "x".repeat(130));
    fs::write(root.join("README.md"), wide).unwrap();
    let agents = Agent::ALL.map(|agent| (Some(agent.variable()), agent.skills()));
    for (variable, skills) in agents.into_iter().chain([(None, ".agentrig/skills")]) {
        let mut lint = runtime(&root, &["lint", "--json"]);
        lint.envs(variable.map(|name| (name, "1")));
        let result = run(&mut lint, "");
        assert_eq!(result.code, Some(1), "{variable:?}: {}", result.output());
        let findings: Vec<Value> = serde_json::from_str(&result.stdout).unwrap();
        let skill = root.join(skills).join("condense-markdown/SKILL.md");
        let named = |finding: &Value| finding["skill"] == skill.to_str().unwrap();
        let guided = !findings.is_empty() && findings.iter().all(named);
        assert!(guided, "{variable:?}: {}", result.stdout);
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
