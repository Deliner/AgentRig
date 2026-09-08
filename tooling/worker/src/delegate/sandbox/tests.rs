use super::{Layout, base, claude, command, prepare};
use crate::delegate::config::{Frontend, Profile};
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

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
        .environment
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
    fs::write(&profile.environment.programs["helper"], "changed").unwrap();
    fs::remove_dir_all(&profile.environment.skills[0]).unwrap();
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
    profile
        .environment
        .programs
        .insert("helper".into(), program);
    profile.environment.skills.push(skill);
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
    profile.environment.programs.insert("hook".into(), script);
    profile.environment.hooks.insert(
        "observe".into(),
        serde_json::from_value(json!({
            "event":"SessionStart", "program":"hook", "args":[argument], "timeout_seconds":10
        }))
        .unwrap(),
    );
    profile
}

#[test]
fn unselected_profile_does_not_inherit_another_environment() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    let selected_layout = fixture(&first);
    let empty_layout = fixture(&second);
    let selected = complete_environment(&first);
    prepare(&selected_layout, &selected).unwrap();
    let empty = profile();
    let receipt = prepare(&empty_layout, &empty).unwrap();
    assert_eq!(receipt["files"], json!({}));
    let settings: toml::Value = toml::from_str(
        &fs::read_to_string(empty_layout.private.join("codex/config.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(settings["features"]["hooks"].as_bool(), Some(false));
    assert!(settings["mcp_servers"].as_table().unwrap().is_empty());
    assert!(settings["hooks"].as_table().unwrap().is_empty());
    fs::write(&empty_layout.codex,
        "#!/bin/sh\nset -eu\ntest ! -e /tools/helper\ntest ! -e /tools/hook\ntest ! -e /codex/skills/guide\ntest ! -e /work/hook-output\n",
    ).unwrap();
    let output = command(&empty_layout, &empty).unwrap().output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        selected_layout
            .private
            .join("environment/skills/guide/SKILL.md")
            .exists()
    );
}

fn complete_environment(root: &Path) -> Profile {
    let mut selected = resources(root);
    selected.environment.mcp_servers.insert(
        "probe".into(),
        serde_json::from_value(json!({"program":"helper"})).unwrap(),
    );
    selected.environment.hooks = hook_profile(root, "instruction").environment.hooks;
    selected
        .environment
        .programs
        .insert("hook".into(), root.join("hook"));
    selected
}

#[test]
fn claude_receives_frozen_skills_hooks_and_mcp_configuration() {
    let root = tempfile::tempdir().unwrap();
    let layout = fixture(root.path());
    let mut profile = complete_environment(root.path());
    profile.frontend = Frontend::ClaudeCode;
    prepare(&layout, &profile).unwrap();
    fs::remove_dir_all(&profile.environment.skills[0]).unwrap();
    fs::write(&profile.environment.programs["helper"], "changed").unwrap();
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(layout.private.join("claude/settings.json")).unwrap())
            .unwrap();
    let mcp: serde_json::Value =
        serde_json::from_slice(&fs::read(layout.private.join("claude/mcp.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["probe"]["command"], "/tools/helper");
    let hook = settings["hooks"]["SessionStart"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    let probe = format!(
        "{}\n{hook}\n! echo changed > /claude/settings.json\n! echo changed > /claude/mcp.json\n! echo changed > /project/input.txt\n",
        resource_probe().replace("/codex/", "/claude/")
    );
    let output = base(&layout, &profile)
        .unwrap()
        .args(["/bin/sh", "-c", &probe])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(layout.private.join("work/hook-output")).unwrap(),
        "instruction"
    );
    assert_eq!(
        fs::read_to_string(layout.input.join("project/input.txt")).unwrap(),
        "original"
    );
}

#[test]
fn claude_response_cannot_follow_a_model_created_result_symlink() {
    let root = tempfile::tempdir().unwrap();
    let layout = fixture(root.path());
    fs::create_dir_all(layout.private.join("work")).unwrap();
    let protected = root.path().join("protected");
    fs::write(&protected, "original").unwrap();
    std::os::unix::fs::symlink(&protected, layout.private.join("work/result.json")).unwrap();
    let output = br#"{"is_error":false,"structured_output":{"ok":true}}"#;
    let error = claude::response(&layout, output).unwrap_err();
    assert!(error.to_string().contains("reserved"));
    assert_eq!(fs::read_to_string(protected).unwrap(), "original");
}
