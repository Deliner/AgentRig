mod settings;

use super::Context;
use crate::shared::testing::{FAIL, GATE, REPAIR, project};
use serde_json::{Value, json};
use std::fs;

const TEST: &str = "test:\n  argv: [sh, -c, exit 0]\n  accepts_args: true\n";

fn rejection(commands: &str, checks: &str) -> String {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, commands, checks);
    let error = Context::load(&root).err().unwrap();
    format!("{error:#}")
}

fn reminder(settings: &Value) -> anyhow::Result<Context> {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, GATE);
    let hooks = format!("discipline_skill: {REPAIR}\nreminder: reminder.json\n");
    fs::write(root.join(".agentrig/hooks.yaml"), hooks).unwrap();
    fs::write(root.join("reminder.json"), settings.to_string()).unwrap();
    Context::load(&root)
}

#[test]
fn reminder_configuration_rejects_malformed_values() {
    let valid = json!({"attention_interval_tokens": 100, "full_refresh_interval_tokens": 300,
        "attention_message": "Reassess"});
    reminder(&valid).unwrap();
    let changes = [
        json!({"attention_message": 17}),
        json!({"attention_message": ""}),
        json!({"unknown_setting": true}),
        json!({"attention_interval_tokens": 0}),
        json!({"full_refresh_interval_tokens": 100}),
    ];
    for change in changes {
        let mut settings = valid.clone();
        let fields = change.as_object().unwrap().clone();
        settings.as_object_mut().unwrap().extend(fields);
        let error = format!("{:#}", reminder(&settings).err().unwrap());
        assert!(error.contains("hooks.reminder"), "{change}: {error}");
    }
}

#[test]
fn affected_groups_reject_invalid_configuration() {
    let groups = [
        "{include: [], targets: []}",
        "{include: ['['], targets: []}",
        "{include: [src/**], targets: ['']}",
        r#"{include: [src/**], targets: ["a\0b"]}"#,
    ];
    for group in groups {
        let checks = format!(
            "- id: tests\n  kind: command\n  command: test\n  skill: {REPAIR}\n  affected:\n  - {group}\n"
        );
        let error = rejection(TEST, &checks);
        assert!(error.contains("affected"), "{group}: {error}");
    }
}

#[test]
fn check_naming_an_unknown_command_is_rejected() {
    let error = rejection(FAIL, &GATE.replace("command: fail", "command: unknown"));
    let named = error.contains("checks.tests: unknown command unknown");
    assert!(named, "{error}");
}
