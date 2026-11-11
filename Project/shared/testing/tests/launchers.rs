use super::{Run, hermetic, invoke, repository, run, search_path};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const LAUNCHER_ENVIRONMENT: [&str; 2] = ["WORKER_BINARY", "WORKER_SOURCE_ROOT"];
const RESOURCES: [&str; 3] = [
    "command_line/main.rs",
    "shared/architecture.yaml",
    "shared/skills/repair/SKILL.md",
];
const FAKE_CARGO: &str = r#"#!/bin/sh
while [ "$#" -gt 0 ]; do
  case "$1" in
    --manifest-path) manifest=$2; shift ;;
    --target-dir) target=$2; shift ;;
  esac
  shift
done
mkdir -p "$target/release"
printf '#!/bin/sh\necho %s\n' "$(cat "${manifest%/*}/RESOURCE")" > "$target/release/agentrig"
chmod +x "$target/release/agentrig"
echo build >> "$target/builds"
"#;

fn launcher(root: &Path, name: &str) -> PathBuf {
    let copy = root.join("tooling/worker").join(name);
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fs::copy(repository().join("tooling/worker").join(name), &copy).unwrap();
    copy
}

fn development(program: &Path) -> Command {
    let mut command = hermetic(program);
    for key in LAUNCHER_ENVIRONMENT {
        command.env_remove(key);
    }
    command
}

fn development_tree(temp: &Path) -> PathBuf {
    let root = temp.canonicalize().unwrap();
    for name in ["run", "build"] {
        launcher(&root, name);
    }
    let pin = root.join("tooling/distribution/stable.txt");
    fs::create_dir_all(pin.parent().unwrap()).unwrap();
    fs::write(pin, format!("{REVISION}\n")).unwrap();
    fs::create_dir_all(root.join("Project")).unwrap();
    let manifest = root.join("Project/Cargo.toml");
    fs::write(manifest, "deliberately invalid manifest\n").unwrap();
    root
}

fn launch(root: &Path, name: &str, args: &[&str], input: &str) -> Run {
    let program = root.join("tooling/worker").join(name);
    run(development(&program).args(args).current_dir(root), input)
}

fn installed_binary(installed: &Path) -> PathBuf {
    let versions = fs::read_dir(installed.join(".agentrig/bin")).unwrap();
    let binaries: Vec<PathBuf> = versions
        .map(|entry| entry.unwrap().path().join("agentrig"))
        .filter(|path| path.is_file())
        .collect();
    assert_eq!(binaries.len(), 1, "{binaries:?}");
    binaries[0].clone()
}

#[test]
fn candidate_build_rebuilds_changed_content_even_with_an_old_mtime() {
    for resource in RESOURCES {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let build = launcher(&root, "build");
        let payload = root.join("Project").join(resource);
        fs::create_dir_all(payload.parent().unwrap()).unwrap();
        fs::write(root.join("Project/Cargo.toml"), "manifest").unwrap();
        fs::write(&payload, "ONE").unwrap();
        let written = fs::metadata(&payload).unwrap().modified().unwrap();
        let cargo = root.join("bin/cargo");
        fs::create_dir_all(cargo.parent().unwrap()).unwrap();
        fs::write(&cargo, FAKE_CARGO.replace("RESOURCE", resource)).unwrap();
        fs::set_permissions(&cargo, fs::metadata(&build).unwrap().permissions()).unwrap();
        let target = root.join("target");
        let path = search_path(root.join("bin"));
        let mut outputs = Vec::new();
        for value in ["ONE", "ONE", "TWO"] {
            let changed = fs::read_to_string(&payload).unwrap() != value;
            if changed {
                fs::write(&payload, value).unwrap();
                let file = fs::File::options().write(true).open(&payload).unwrap();
                file.set_modified(written).unwrap();
            }
            let mut command = development(&build);
            command.env("PATH", &path).env("CARGO_TARGET_DIR", &target);
            let result = run(&mut command, "");
            assert_eq!(result.code, Some(0), "{resource}: {}", result.output());
            outputs.push(result.stdout.trim().to_owned());
        }
        assert_eq!(outputs, ["ONE", "ONE", "TWO"], "{resource}");
        let builds = fs::read_to_string(target.join("builds")).unwrap();
        assert_eq!(builds, "build\nbuild\n", "{resource}");
    }
}

#[test]
fn stable_runtime_survives_a_candidate_build_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = development_tree(temp.path());
    let installed = root.join(".cache/development").join(REVISION);
    fs::create_dir_all(&installed).unwrap();
    let init = invoke(&installed, &["init"]);
    assert_eq!(init.code, Some(0), "{}", init.output());
    let binary = installed_binary(&installed);
    let before = fs::read(&binary).unwrap();
    let candidate = launch(&root, "build", &["--version"], "");
    assert_ne!(candidate.code, Some(0), "{}", candidate.output());
    assert!(
        candidate.stderr.contains("Cargo.toml"),
        "{}",
        candidate.output()
    );
    let installed_root = installed.to_str().unwrap();
    for args in [
        &["--version"][..],
        &["config-check", "--root", installed_root],
    ] {
        let result = launch(&root, "run", args, "");
        assert_eq!(result.code, Some(0), "{args:?}: {}", result.output());
    }
    let event = r#"{"hook_event_name":"SessionStart"}"#;
    let hook = launch(&root, "run", &["hook", "--root", installed_root], event);
    assert_eq!(hook.code, Some(0), "{}", hook.output());
    assert!(
        hook.stdout.contains("complexity-discipline"),
        "{}",
        hook.stdout
    );
    assert_eq!(fs::read(&binary).unwrap(), before);
}

#[test]
fn missing_stable_runtime_never_builds_the_candidate() {
    let temp = tempfile::tempdir().unwrap();
    let root = development_tree(temp.path());
    let result = launch(&root, "run", &["--version"], "");
    assert_eq!(result.code, Some(2), "{}", result.output());
    assert!(
        result.stderr.contains("just bootstrap"),
        "{}",
        result.stderr
    );
    assert!(!root.join(".cache/worker").exists());
}
