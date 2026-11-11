use crate::project_setup::initialization::init;
use crate::shared::testing::{hermetic, invoke, run, snapshot};
use serde_json::{Value, json};
use std::{fs, path::Path};

const FRONTEND: [&str; 2] = ["--frontend", "claude-code"];

fn json_file(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn claude_init_registers_guarded_hooks() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &[&["init"][..], &FRONTEND].concat());
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    assert!(!root.join(".codex").exists());
    assert!(!root.join(".mcp.json").exists());
    let note = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert_eq!(note, "@AGENTS.md\n");
    let settings = json_file(&root.join(".claude/settings.json"));
    let command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"].as_str();
    let event = json!({"hook_event_name": "PreToolUse", "tool_name": "Bash",
        "tool_input": {"command": "echo bypass"}, "cwd": root});
    let mut hook = hermetic("sh");
    hook.args(["-c", command.unwrap()]).current_dir(&root);
    let result = run(&mut hook, &event.to_string());
    assert_eq!(result.code, Some(0), "{}", result.output());
    let output: Value = serde_json::from_str(&result.stdout).unwrap();
    let decision = &output["hookSpecificOutput"]["permissionDecision"];
    assert_eq!(decision, "deny", "{output}");
}

#[test]
fn claude_init_collision_preserves_files() {
    let frontend = FRONTEND.map(str::to_owned);
    for existing in [".claude/settings.json", "CLAUDE.md"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join(existing);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{\"model\": \"consumer-model\"}\n").unwrap();
        let before = snapshot(&root);
        let error = init(&root, &frontend).unwrap_err().to_string();
        assert!(error.contains("collision"), "{existing}: {error}");
        assert_eq!(snapshot(&root), before, "{existing}");
    }
}
