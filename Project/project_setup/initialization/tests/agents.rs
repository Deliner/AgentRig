// DECISION: D188
use crate::project_setup::initialization::init;
use crate::shared::calling_agent::Agent;
use crate::shared::testing::{hermetic, initialized, repository, run, runtime, snapshot, write};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

const CLIENT_FILES: [&str; 4] = [
    ".codex/config.toml",
    ".codex/hooks.json",
    ".claude/settings.json",
    "CLAUDE.md",
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

// DECISION: D195
#[test]
fn init_and_the_rig_give_both_agents_one_rules_text() {
    let (_temp, root) = initialized();
    for project in [root.as_path(), repository()] {
        let text = |name: &str| fs::read_to_string(project.join(name)).unwrap();
        let shown = project.display();
        assert_eq!(text("CLAUDE.md"), text("AGENTS.md"), "{shown}");
        let imported = project.join(".claude/CLAUDE.md").exists();
        assert!(!imported, "{shown}");
    }
}

#[test]
fn init_registers_both_agents_and_each_sees_skills_through_its_directory() {
    let (_temp, root) = initialized();
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
    let (_temp, root) = initialized();
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
    let (_temp, root) = initialized();
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
        write(&root, existing, "{\"model\": \"consumer-model\"}\n");
        let before = snapshot(&root);
        let error = init(&root, &[]).unwrap_err().to_string();
        assert!(error.contains("collision"), "{existing}: {error}");
        assert_eq!(snapshot(&root), before, "{existing}");
    }
}
