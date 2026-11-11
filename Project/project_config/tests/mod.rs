use super::Context;
use crate::shared::testing::{FAIL, GATE, REPAIR, project};

const TEST: &str = "test:\n  argv: [sh, -c, exit 0]\n  accepts_args: true\n";

fn rejection(commands: &str, checks: &str) -> String {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, commands, checks);
    let error = Context::load(&root).err().unwrap();
    format!("{error:#}")
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
