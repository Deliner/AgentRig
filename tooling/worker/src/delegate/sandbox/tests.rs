use super::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;

fn profile() -> Profile {
    serde_json::from_value(
        json!({"frontend":"codex","model":"fixture","reasoning_effort":"high",
        "mode":"artifacts","prompt":"unused","visible_paths":[],"timeout_seconds":60,
        "credentials":{"env":{}}}),
    )
    .unwrap()
}

#[test]
fn sandbox_restricts_filesystem_and_environment_with_real_bubblewrap() {
    let root = tempfile::tempdir().unwrap();
    let layout = fixture(root.path());
    let mut profile = profile();
    profile
        .programs
        .insert("printf".into(), "/usr/bin/printf".into());
    prepare(&layout, &profile).unwrap();
    let output = command(&layout, &profile).unwrap().output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(layout.input.join("project/input.txt")).unwrap(),
        "original"
    );
    assert_eq!(
        fs::read_to_string(layout.private.join("work/result.json")).unwrap(),
        r#"{"ok":true}"#
    );
}

fn fixture(root: &Path) -> Layout {
    let layout = Layout {
        input: root.join("input"),
        private: root.join("private"),
        codex: root.join("codex"),
    };
    fs::create_dir(&layout.input).unwrap();
    for name in ["project", "inputs"] {
        fs::create_dir(layout.input.join(name)).unwrap();
    }
    fs::write(layout.input.join("project/input.txt"), "original").unwrap();
    fs::write(root.join("codex-code-mode-host"), []).unwrap();
    let script = format!(
        "#!/bin/sh\nset -eu\ntest ! -e '{}'\ntest ! -e /home/deliner\ntest ! -e /usr/bin/python3\ntest ! -e /run/user\ntest -z \"${{UNDECLARED_SECRET:-}}\"\ntest \"$HOME\" = /home/delegate\n! echo changed > /project/input.txt\n! echo changed > /codex/config.toml\n/tools/printf '{{\"ok\":true}}' > /work/result.json\n",
        root.display()
    );
    fs::write(&layout.codex, script).unwrap();
    fs::set_permissions(&layout.codex, fs::Permissions::from_mode(0o755)).unwrap();
    layout
}
