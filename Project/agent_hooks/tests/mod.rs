mod input;
mod memory;
mod named_commands;
mod protected;
mod reminders;
mod shell;
mod skill_requirements;
mod tools;

use super::dispatch;
use crate::shared::calling_agent::Agent;
use serde_json::{Value, json};
use std::{fs, path::Path};

const PROTECTED_CONFIGURATION: [&str; 3] = ["project.yaml", "checks.yaml", "hooks.yaml"];
const PROTECTED_LIST: &str = ".agentrig/project.yaml, .agentrig/git.yaml, .agentrig/commands.yaml, .agentrig/checks.yaml, .agentrig/hooks.yaml, .agentrig/tools.yaml, .agentrig/rust.yaml";

fn initialized(args: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let args: Vec<String> = args.iter().copied().map(str::to_owned).collect();
    crate::project_setup::init(&root, &args).unwrap();
    (temp, root)
}

fn project() -> (tempfile::TempDir, std::path::PathBuf) {
    initialized(&[])
}

fn pre_tool(cwd: &Path, tool: &str, input: Value) -> Value {
    json!({"hook_event_name": "PreToolUse", "tool_name": tool,
        "tool_input": input, "cwd": cwd.to_str()})
}

fn respond(root: &Path, event: &Value) -> Value {
    dispatch(root, event, Agent::Codex)
        .unwrap()
        .unwrap_or(Value::Null)
}

fn hook(root: &Path, tool: &str, input: Value) -> Value {
    respond(root, &pre_tool(root, tool, input))
}

fn reason(output: &Value) -> &str {
    output["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap_or_default()
}

fn edit(root: &Path, path: &str) -> String {
    reason(&hook(root, "Write", json!({"file_path": path}))).to_owned()
}

fn shell(root: &Path, command: &str) -> String {
    reason(&hook(root, "Bash", json!({"command": command}))).to_owned()
}

#[test]
fn invalid_configuration_leaves_only_its_repair_and_check_available() {
    let (_temp, root) = project();
    let oracles = root.join(".agentrig/oracles.yaml");
    let valid = fs::read_to_string(&oracles).unwrap();
    fs::write(&oracles, "I001:\n  surprise: 1\n").unwrap();
    assert_eq!(edit(&root, ".agentrig/oracles.yaml"), "");
    assert_eq!(edit(&root, &oracles.display().to_string()), "");
    assert_eq!(shell(&root, "just config-check"), "");
    let denied = edit(&root, "Project/main.py");
    let repair = format!(
        "ACTION: Apply {}/.agents/skills/repair/SKILL.md",
        root.display()
    );
    let protected = format!("protected {PROTECTED_LIST};");
    for expected in [
        ".agentrig/oracles.yaml: oracles.I001",
        &repair,
        "only edits of .agentrig/oracles.yaml, .agentrig/lint.yaml",
        &protected,
        "`just config-check`",
    ] {
        assert!(denied.contains(expected), "{expected}: {denied}");
    }
    assert_eq!(edit(&root, ".agentrig/checks.yaml"), denied);
    for command in ["just run test", "just config-check; touch x", "just list"] {
        assert_eq!(shell(&root, command), denied, "{command}");
    }
    let patch = "*** Update File: .agentrig/oracles.yaml\n*** Add File: Project/x.py\n";
    assert_eq!(reason(&hook(&root, "apply_patch", json!(patch))), denied);
    let codex = "*** Begin Patch\n*** Update File: .agentrig/oracles.yaml\n*** End Patch";
    assert_eq!(
        reason(&hook(&root, "apply_patch", json!({"command": codex}))),
        ""
    );
    let session = respond(&root, &json!({"hook_event_name": "SessionStart"}));
    assert_eq!(session["hookSpecificOutput"]["additionalContext"], denied);
    fs::write(&oracles, valid).unwrap();
    assert_eq!(edit(&root, "README.md"), "");
    assert_eq!(shell(&root, "just run test"), "");
}

#[test]
fn invalid_configuration_can_be_read_only_through_cat_of_its_files() {
    let (_temp, root) = project();
    fs::write(root.join(".agentrig/commands.yaml"), "read:\n  argv: []\n").unwrap();
    let read = "cat .agentrig/commands.yaml '.agentrig/skills/repair/SKILL.md'";
    assert_eq!(shell(&root, read), "");
    let denied = edit(&root, "Project/main.py");
    let repair = format!("{}/.agents/skills/repair/SKILL.md", root.display());
    assert!(
        denied.contains(&format!(
            "`cat` of these files, {PROTECTED_LIST} and {repair}"
        )),
        "{denied}"
    );
    assert_eq!(shell(&root, &format!("cat {repair}")), "");
    for command in [
        "cat Project/main.py",
        "cat .agentrig/commands.yaml > Project/main.py",
        "sed -n 1p .agentrig/commands.yaml",
        "cat",
    ] {
        assert_eq!(shell(&root, command), denied, "{command}");
    }
}

#[test]
fn diagnostics_distinguish_missing_corrupt_unknown_and_invalid_configuration() {
    let cases = [
        ("project.yaml", None, "No such file or directory"),
        ("project.yaml", Some("version: [1\n"), "YAML configuration"),
        (
            "hooks.yaml",
            Some("surprise: 1\n"),
            ".agentrig/hooks.yaml: hooks.surprise: unknown field",
        ),
        (
            "checks.yaml",
            Some("- id: x\n  kind: memory\n  skill: none.md\n"),
            ".agentrig/checks.yaml: checks.x.skill",
        ),
        (
            "lint.yaml",
            Some("version: 1\nextra: 1\n"),
            "paths.lint: YAML configuration",
        ),
    ];
    for (file, content, expected) in cases {
        let (_temp, root) = project();
        let path = root.join(".agentrig").join(file);
        match content {
            Some(content) => fs::write(&path, content).unwrap(),
            None => fs::remove_file(&path).unwrap(),
        }
        let denied = edit(&root, "Project/main.py");
        assert!(denied.contains(expected), "{file}: {denied}");
        assert!(!denied.contains("required"), "{file}: {denied}");
        let protected = PROTECTED_CONFIGURATION.contains(&file);
        let repair = if protected { denied.as_str() } else { "" };
        assert_eq!(edit(&root, &format!(".agentrig/{file}")), repair, "{file}");
    }
}

#[test]
fn referenced_lint_and_reminder_files_are_repairable() {
    let (_temp, root) = project();
    let reminder = root.join(".agentrig/skills/complexity-discipline/context-reminder.json");
    fs::write(&reminder, "{").unwrap();
    let denied = edit(&root, "Project/main.py");
    assert!(denied.contains("hooks.reminder JSON"), "{denied}");
    assert!(denied.contains(
        ".agentrig/lint.yaml, .agentrig/skills/complexity-discipline/context-reminder.json"
    ));
    assert_eq!(
        edit(
            &root,
            ".agentrig/skills/complexity-discipline/context-reminder.json"
        ),
        ""
    );
    assert_eq!(edit(&root, ".agentrig/hooks/pre-commit"), denied);
}
