use super::installed_runtime;
use crate::shared::{
    fs::{Permissions, PermissionsExt},
    testing::{Run, git, hermetic, invoke, launch, run, search_path, worker},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const GUARDED: &str = "direct commits on main are prohibited";

fn initialized() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap().join("project");
    fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    (temp, root)
}

fn pre_commit(root: &Path, system: &str) -> Run {
    let host = root.with_file_name("host");
    fs::create_dir_all(&host).unwrap();
    let uname = host.join("uname");
    fs::write(&uname, format!("#!/bin/sh\necho {system}\n")).unwrap();
    fs::set_permissions(&uname, Permissions::from_mode(0o755)).unwrap();
    let mut hook = hermetic(root.join(".agentrig/hooks/pre-commit"));
    hook.current_dir(root).env("PATH", search_path(host));
    run(&mut hook, "")
}

// DECISION: D150
#[test]
fn hooks_run_the_runtime_built_for_the_reported_platform() {
    let (_temp, root) = initialized();
    let mut runtime = installed_runtime(&root).parent().unwrap().to_owned();
    let platforms = [
        ("Darwin arm64", "macos-arm64"),
        ("Linux x86_64", "linux-x86_64"),
    ];
    for (system, platform) in platforms {
        let renamed = runtime.with_file_name(platform);
        fs::rename(&runtime, &renamed).unwrap();
        runtime = renamed;
        let result = pre_commit(&root, system);
        assert_eq!(result.code, Some(1), "{system}: {}", result.stderr);
        assert!(
            result.stderr.contains(GUARDED),
            "{system}: {}",
            result.stderr
        );
    }
}

// DECISION: D150
#[test]
fn hooks_without_a_runtime_for_the_platform_name_it_and_the_add_command() {
    let (_temp, root) = initialized();
    let result = pre_commit(&root, "Linux riscv64");
    assert_eq!(result.code, Some(2), "{}", result.stderr);
    let named = result.stderr.contains("no runtime for linux-riscv64")
        && result.stderr.contains("agentrig runtime-add");
    assert!(named, "{}", result.stderr);
}

// DECISION: D150
#[test]
fn runtime_add_moves_a_single_binary_project_to_platform_runtimes() {
    let (_temp, root) = initialized();
    let runtime = installed_runtime(&root);
    let launcher = root.join(".agentrig/bin/agentrig");
    fs::rename(&runtime, &launcher).unwrap();
    fs::remove_dir(runtime.parent().unwrap()).unwrap();
    git(&root, &["add", "."]);
    let unhooked = [
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "-qm",
        "single binary",
    ];
    git(&root, &unhooked);
    git(&root, &["switch", "-c", "feature/runtime"]);
    let added = invoke(&root, &["runtime-add"]);
    assert_eq!(added.code, Some(0), "{}", added.output());
    let script = fs::read_to_string(&launcher).unwrap();
    assert!(script.starts_with("#!/bin/sh\n"), "{script}");
    assert!(fs::read(&runtime).unwrap() == fs::read(worker()).unwrap());
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "platform runtimes"]);
    let merged = run(&mut launch(&launcher, &root, &["feature-merge"]), "");
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    assert_eq!(git(&root, &["branch", "--show-current"]), "main");
    let diagnosed = run(&mut launch(&launcher, &root, &["doctor"]), "");
    assert_eq!(diagnosed.code, Some(0), "{}", diagnosed.output());
}
