// DECISION: D174
use super::write;
use crate::shared::fs_with_context::{Permissions, PermissionsExt};
use crate::shared::testing::{Run, repository, run, runtime, search_path};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n";

fn rig_version() -> String {
    let text = fs::read_to_string(repository().join("rust-toolchain.toml")).unwrap();
    let file: toml::Table = toml::from_str(&text).unwrap();
    file["toolchain"]["channel"].as_str().unwrap().to_owned()
}

fn older(version: &str) -> String {
    let parts: Vec<u32> = version
        .split('.')
        .map(|part| part.parse().unwrap())
        .collect();
    format!("{}.{}.99", parts[0], parts[1] - 1)
}

fn rustc(directory: &Path, version: &str) -> OsString {
    let bin = directory.join(version);
    fs::create_dir_all(&bin).unwrap();
    let path = bin.join("rustc");
    let script = format!("#!/bin/sh\necho 'rustc {version} (0123456 2026-11-12)'\n");
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, Permissions::from_mode(0o755)).unwrap();
    search_path(bin)
}

fn rust_project(temp: &Path) -> PathBuf {
    let root = temp.join("project");
    write(&root, "Cargo.toml", MANIFEST);
    write(&root, "src/main.rs", "fn main() {}\n");
    root.canonicalize().unwrap()
}

fn agentrig(root: &Path, path: &OsString, args: &[&str]) -> Run {
    run(runtime(root, args).env("PATH", path), "")
}

fn entries(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn rust_init_refuses_an_older_or_missing_rust_before_writing_files() {
    let temp = tempfile::tempdir().unwrap();
    let required = rig_version();
    let older = older(&required);
    let empty = temp.path().join("empty");
    fs::create_dir_all(&empty).unwrap();
    let cases = [
        (rustc(temp.path(), &older), format!("found {older}")),
        (empty.into_os_string(), "found no rustc version".to_owned()),
    ];
    for (path, found) in cases {
        let project = tempfile::tempdir().unwrap();
        let root = rust_project(project.path());
        let result = agentrig(&root, &path, &["init", "--language", "rust"]);
        let output = result.output();
        assert_eq!(result.code, Some(2), "{output}");
        let requires = format!("requires {required} or newer");
        for expected in [
            found.as_str(),
            &requires,
            "rustup update",
            "rust-toolchain.toml",
        ] {
            assert!(output.contains(expected), "{expected}: {output}");
        }
        assert_eq!(entries(&root), ["Cargo.toml", "src"], "{output}");
    }
}

#[test]
fn rust_init_accepts_the_rig_rust_and_doctor_tells_it_from_an_older_one() {
    let temp = tempfile::tempdir().unwrap();
    let required = rig_version();
    let older = older(&required);
    let root = rust_project(temp.path());
    let current = rustc(temp.path(), &required);
    let initialized = agentrig(&root, &current, &["init", "--language", "rust"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let accepted = agentrig(&root, &current, &["doctor"]);
    let line = format!("rust toolchain: found {required}, requires {required} or newer\n");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    assert!(accepted.stdout.contains(&line), "{}", accepted.output());
    let refused = agentrig(&root, &rustc(temp.path(), &older), &["doctor"]);
    let line = format!("rust toolchain: OLDER OR MISSING, found {older}, requires {required}");
    assert_eq!(refused.code, Some(1), "{}", refused.output());
    assert!(refused.stdout.contains(&line), "{}", refused.output());
}

#[test]
fn python_init_and_doctor_ignore_the_rust_version() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let path = rustc(temp.path(), &older(&rig_version()));
    let initialized = agentrig(&root, &path, &["init", "--language", "python"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let doctor = agentrig(&root, &path, &["doctor"]);
    assert_eq!(doctor.code, Some(0), "{}", doctor.output());
    assert!(
        !doctor.stdout.contains("rust toolchain"),
        "{}",
        doctor.stdout
    );
}
