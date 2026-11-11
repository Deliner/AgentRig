use crate::project_config::VERSION;
use crate::shared::{
    fs::{Permissions, PermissionsExt},
    testing::{git, invoke, rerun},
};
use std::{fs, path::Path};

const CHANGES: [(&str, &str, &str, &str); 5] = [
    (
        ".codex/config.toml",
        "hooks = true",
        "hooks = false",
        "Codex registration",
    ),
    (
        ".codex/hooks.json",
        "SessionStart",
        "UnknownEvent",
        "Codex registration",
    ),
    (
        ".agentrig/hooks/pre-commit",
        "--staged",
        "--incorrect",
        "git hooks",
    ),
    (
        ".agentrig/commands.yaml",
        "python3",
        "missing-tool-xyz",
        "MISSING",
    ),
    (".agentrig/project.yaml", VERSION, "999.0.0", "project pins"),
];
const INCOMPATIBLE: &str = "installed binary: MISSING OR INCOMPATIBLE";

fn doctor(root: &Path) -> Option<i32> {
    invoke(root, &["doctor"]).code
}

fn diagnosed(root: &Path, expected: &str) {
    let result = invoke(root, &["doctor"]);
    let output = result.output();
    assert_ne!(result.code, Some(0), "{output}");
    assert!(output.contains(expected), "{expected}: {output}");
    let guided = result.stderr.contains(".agentrig/skills/repair/SKILL.md");
    assert!(guided, "{output}");
    let repeated = rerun(&result.stderr);
    assert_eq!(repeated.code, result.code, "{}", repeated.output());
    let reproduced = repeated.output().contains(expected);
    assert!(reproduced, "{expected}: {}", repeated.output());
}

fn executable(path: &Path, mode: u32) {
    fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
}

#[test]
fn doctor_observes_registration_and_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q"]);
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    assert_eq!(doctor(&root), Some(0));
    for (name, before, after, expected) in CHANGES {
        let path = root.join(name);
        let original = fs::read_to_string(&path).unwrap();
        assert!(original.contains(before), "{name}: {original}");
        fs::write(&path, original.replace(before, after)).unwrap();
        diagnosed(&root, expected);
        fs::write(&path, original).unwrap();
    }
    let hook = root.join(".agentrig/hooks/pre-commit");
    executable(&hook, 0o644);
    assert_eq!(doctor(&root), Some(1));
    executable(&hook, 0o755);
    let binary = root.join(".agentrig/bin/agentrig");
    fs::write(&binary, "#!/bin/sh\necho agentrig 999.0.0\n").unwrap();
    let result = invoke(&root, &["doctor"]);
    assert_eq!(result.code, Some(1), "{}", result.output());
    assert!(result.stdout.contains(INCOMPATIBLE), "{}", result.stdout);
}
