use super::dispatch;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn project() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    crate::project_setup::setup::init(&root, &[]).unwrap();
    (temp, root)
}

fn hook(root: &Path, tool: &str, input: Value) -> Value {
    let event = json!({"hook_event_name": "PreToolUse", "tool_name": tool,
        "tool_input": input, "cwd": root.to_str()});
    dispatch(root, &event).unwrap().unwrap_or(Value::Null)
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
    let commands = root.join(".agentrig/commands.yaml");
    let valid = fs::read_to_string(&commands).unwrap();
    fs::write(&commands, "read:\n  argv: []\n").unwrap();
    assert_eq!(edit(&root, ".agentrig/commands.yaml"), "");
    assert_eq!(edit(&root, &commands.display().to_string()), "");
    assert_eq!(shell(&root, "just config-check"), "");
    let denied = edit(&root, "Project/main.py");
    for expected in [
        ".agentrig/commands.yaml: commands.read: empty argv requires accepts_args",
        "ACTION: Apply .agentrig/skills/repair/SKILL.md",
        "only edits of .agentrig/project.yaml, .agentrig/git.yaml",
        "`just config-check`",
    ] {
        assert!(denied.contains(expected), "{expected}: {denied}");
    }
    for command in ["just run test", "just config-check; touch x", "just list"] {
        assert_eq!(shell(&root, command), denied, "{command}");
    }
    let patch = "*** Update File: .agentrig/commands.yaml\n*** Add File: Project/x.py\n";
    assert_eq!(reason(&hook(&root, "apply_patch", json!(patch))), denied);
    let codex = "*** Begin Patch\n*** Update File: .agentrig/commands.yaml\n*** End Patch";
    assert_eq!(
        reason(&hook(&root, "apply_patch", json!({"command": codex}))),
        ""
    );
    let session = dispatch(&root, &json!({"hook_event_name": "SessionStart"})).unwrap();
    assert_eq!(
        session.unwrap()["hookSpecificOutput"]["additionalContext"],
        denied
    );
    fs::write(&commands, valid).unwrap();
    assert_eq!(edit(&root, "Project/main.py"), "");
    assert_eq!(shell(&root, "just run test"), "");
}

#[test]
fn invalid_configuration_can_be_read_only_through_cat_of_its_files() {
    let (_temp, root) = project();
    fs::write(root.join(".agentrig/commands.yaml"), "read:\n  argv: []\n").unwrap();
    let read = "cat .agentrig/commands.yaml '.agentrig/skills/repair/SKILL.md'";
    assert_eq!(shell(&root, read), "");
    let denied = edit(&root, "Project/main.py");
    assert!(denied.contains("`cat` of these files and .agentrig/skills/repair/SKILL.md"));
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
        assert_eq!(edit(&root, &format!(".agentrig/{file}")), "", "{file}");
    }
}

#[test]
fn referenced_lint_and_reminder_files_are_repairable() {
    let (_temp, root) = project();
    let reminder = root.join(".agentrig/reminder.json");
    fs::write(&reminder, "{").unwrap();
    let denied = edit(&root, "Project/main.py");
    assert!(denied.contains("hooks.reminder JSON"), "{denied}");
    assert!(denied.contains(".agentrig/lint.yaml, .agentrig/reminder.json"));
    assert_eq!(edit(&root, ".agentrig/reminder.json"), "");
    assert_eq!(edit(&root, ".agentrig/hooks/pre-commit"), denied);
}
