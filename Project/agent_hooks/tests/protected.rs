// DECISION: D213
// DECISION: D214
use super::{edit, hook, project, reason, shell};
use crate::shared::testing::configure;
use serde_json::{Value, json};
use std::path::Path;

const REFUSAL: &str = "protected enforcement files";
// DECISION: D230
// DECISION: D231
const ENFORCEMENT: [&str; 15] = [
    ".agentrig/bin/agentrig",
    ".agentrig/commands.yaml",
    ".agentrig/checks.yaml",
    ".agentrig/hooks/pre-commit",
    ".agentrig/mods/self-compact/hooks/register.ts",
    ".cache/runtime/compaction",
    ".git/config",
    ".agentrig/just/x.just",
    "justfile",
    ".claude/settings.json",
    ".codex/hooks.json",
    ".codex/config.toml",
    ".cache/runtime/reminders/x",
    ".cache/runtime/approved",
    ".cache/runtime/consent",
];

fn delivered(root: &Path, tool: &str, input: Value) -> String {
    let _delivered_route_stops = hook(root, tool, input.clone());
    reason(&hook(root, tool, input)).to_owned()
}

#[test]
fn edits_and_writes_of_enforcement_are_refused_alike_with_the_proposal_skill() {
    let (_temp, root) = project();
    let skill = format!(
        "{}/.agents/skills/propose-protected-change/SKILL.md",
        root.display()
    );
    for path in ENFORCEMENT {
        let denied = edit(&root, path);
        assert!(denied.contains(path), "{path}: {denied}");
        assert!(denied.contains(&skill), "{path}: {denied}");
        let written = shell(&root, &format!("just write -- touch {path}"));
        assert_eq!(written, denied, "{path}");
    }
    let hook_file = edit(&root, ".agentrig/hooks/pre-commit");
    let executable = "just write -- chmod -x .agentrig/hooks/pre-commit";
    assert_eq!(shell(&root, executable), hook_file);
    for command in [
        "just write -- rm -r .agentrig",
        "just write -- cp x --target-directory=.agentrig/just",
        "just write -- mv x ./.agentrig/bin/*",
        "just write -- touch ~/x",
    ] {
        assert!(shell(&root, command).contains(REFUSAL), "{command}");
    }
}

#[test]
fn git_stages_enforcement_but_neither_restores_nor_removes_it() {
    let (_temp, root) = project();
    for command in [
        "just write -- git checkout HEAD -- .agentrig/commands.yaml",
        "just write -- git restore .",
        "just write -- git rm --cached justfile",
        "just write -- git mv .agentrig/just x",
    ] {
        assert!(shell(&root, command).contains(REFUSAL), "{command}");
    }
    for command in [
        "just write -- git add .agentrig/commands.yaml justfile",
        "just write -- git checkout -b feature/x",
    ] {
        let input = json!({"command": command});
        assert_eq!(delivered(&root, "Bash", input), "", "{command}");
    }
}

// DECISION: D226
#[test]
fn only_the_user_approves_or_consents() {
    let (_temp, root) = project();
    for command in ["approve", "consent"] {
        let refused = shell(&root, &format!("just {command}"));
        let asked = format!("`! just {command}`");
        assert!(refused.contains(&asked), "{refused}");
    }
}

// DECISION: D226
#[test]
fn a_proposal_the_lint_configuration_and_ordinary_files_stay_editable() {
    let (_temp, root) = project();
    for path in [
        ".cache/runtime/proposal/.agentrig/commands.yaml",
        ".agentrig/lint.yaml",
        // DECISION: D235
        ".agentrig/lint-baseline.yaml",
        ".agentrig/oracles.yaml",
        "README.md",
    ] {
        let input = json!({"file_path": path});
        assert_eq!(delivered(&root, "Write", input), "", "{path}");
    }
    let proposal = "just write -- mkdir -p .cache/runtime/proposal/.agentrig";
    let input = json!({"command": proposal});
    assert_eq!(delivered(&root, "Bash", input), "");
}

#[test]
fn configured_protection_covers_the_pin_and_listed_paths() {
    let (_temp, root) = project();
    configure(&root, "git", "pin: tooling/stable.txt\n");
    configure(&root, "hooks", "protected: [tooling/worker]\n");
    for path in ["tooling/stable.txt", "tooling/worker/run", "tooling"] {
        assert!(edit(&root, path).contains(REFUSAL), "{path}");
    }
    assert_eq!(edit(&root, "tooling/other"), "");
}

#[test]
fn an_unresolvable_word_or_protection_outside_the_project_refuses() {
    let (_temp, root) = project();
    std::os::unix::fs::symlink("cycle", root.join("cycle")).unwrap();
    let cycle = shell(&root, "just write -- touch cycle/x");
    assert!(cycle.contains(REFUSAL), "{cycle}");
    assert!(cycle.contains("symlink cycle"), "{cycle}");
    configure(&root, "hooks", "protected: [../outside]\n");
    let denied = edit(&root, "README.md");
    assert!(denied.contains("hooks.protected"), "{denied}");
}
