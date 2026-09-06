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

#[test]
fn sandbox_uses_frozen_programs_and_skill_trees() {
    let root = tempfile::tempdir().unwrap();
    let layout = fixture(root.path());
    let profile = resources(root.path());
    let receipt = prepare(&layout, &profile).unwrap();
    fs::write(&profile.programs["helper"], "changed").unwrap();
    fs::remove_dir_all(&profile.skills[0]).unwrap();
    fs::write(&layout.codex, resource_probe()).unwrap();
    let output = command(&layout, &profile).unwrap().output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        receipt["files"]["skills/guide/SKILL.md"]["sha256"],
        crate::resources::digest(b"original guide\n")
    );
    assert_eq!(
        receipt["files"]["skills/guide/support/run"]["executable"],
        true
    );
    assert_eq!(
        receipt["files"]["skills/guide/SKILL.md"]["executable"],
        false
    );
    assert_eq!(receipt["files"]["programs/helper"]["executable"], true);
}

fn resources(root: &Path) -> Profile {
    let mut profile = profile();
    let program = root.join("helper");
    fs::write(&program, "#!/bin/sh\nprintf original\n").unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    let skill = root.join("guide");
    fs::create_dir_all(skill.join("support")).unwrap();
    fs::write(skill.join("SKILL.md"), "original guide\n").unwrap();
    fs::copy(&program, skill.join("support/run")).unwrap();
    profile.programs.insert("helper".into(), program);
    profile.skills.push(skill);
    profile
}

fn resource_probe() -> &'static str {
    "#!/bin/sh\nset -eu
test \"$(/tools/helper)\" = original
test \"$(/codex/skills/guide/support/run)\" = original
read -r guide < /codex/skills/guide/SKILL.md
test \"$guide\" = 'original guide'
! echo changed > /tools/helper
! echo changed > /codex/skills/guide/SKILL.md
test ! -x /codex/skills/guide/SKILL.md
"
}

#[test]
fn configured_hooks_use_read_only_programs_and_literal_arguments() {
    let root = tempfile::tempdir().unwrap();
    let layout = fixture(root.path());
    let argument = "spaces 'quotes' $(echo expansion) ; exit 1";
    let profile = hook_profile(root.path(), argument);
    prepare(&layout, &profile).unwrap();
    let value: toml::Value =
        toml::from_str(&fs::read_to_string(layout.private.join("codex/config.toml")).unwrap())
            .unwrap();
    assert_eq!(value["features"]["hooks"].as_bool(), Some(true));
    let hook = value["hooks"]["SessionStart"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    let output = base(&layout, &profile)
        .unwrap()
        .args(["/bin/sh", "-c", hook])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(layout.private.join("work/hook-output")).unwrap(),
        argument
    );
    assert!(
        command(&layout, &profile)
            .unwrap()
            .get_args()
            .any(|arg| arg == "--dangerously-bypass-hook-trust")
    );
}

fn hook_profile(root: &Path, argument: &str) -> Profile {
    let mut profile = profile();
    let script = root.join("hook");
    fs::write(
        &script,
        "#!/bin/sh\nprintf '%s' \"$1\" > /work/hook-output\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    profile.programs.insert("hook".into(), script);
    profile.hooks.insert(
        "observe".into(),
        serde_json::from_value(json!({
            "event":"SessionStart", "program":"hook", "args":[argument], "timeout_seconds":10
        }))
        .unwrap(),
    );
    profile
}
