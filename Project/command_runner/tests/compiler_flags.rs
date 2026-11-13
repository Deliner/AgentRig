use crate::shared::testing::{REPAIR, Run, hermetic, project, run, runtime};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const LINTS: [&str; 7] = [
    "clippy",
    "--quiet",
    "--",
    "-D",
    "warnings",
    "-D",
    "clippy::unwrap_used",
];
const FLAWED: &str = "fn main() {
    let unused = 1;
    let first = std::env::args().next().unwrap();
    println!(\"{first}\");
}
";
const CONDITIONAL: &str = "#[cfg(not(rig_flag))]
compile_error!(\"rig_flag is unset\");
fn main() {}
";
const CAPPED: &str = "[build]\nrustflags = [\"--cap-lints\", \"allow\"]\n";
const FLAGGED: &str = "rustflags: [--cfg, rig_flag, --check-cfg, cfg(rig_flag)]\n";

fn cargo_project(source: &str) -> (TempDir, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let outer = temp.path().canonicalize().unwrap();
    let (root, home) = (outer.join("project"), outer.join("home"));
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&home).unwrap();
    let argv: Vec<&str> = ["cargo"].into_iter().chain(LINTS).collect();
    let commands = format!("clippy:\n  argv: {}\n", json!(argv));
    let checks = format!("- id: clippy\n  kind: command\n  command: clippy\n  skill: {REPAIR}\n");
    project(&root, &commands, &checks);
    fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("src/main.rs"), source).unwrap();
    (temp, root, home)
}

fn cargo(root: &Path, home: &Path, environment: &[(&str, &str)]) -> Run {
    let mut command = hermetic("cargo");
    command
        .args(LINTS)
        .current_dir(root)
        .env("CARGO_HOME", home);
    run(command.envs(environment.iter().copied()), "")
}

fn gate(root: &Path, home: &Path, environment: &[(&str, &str)]) -> Run {
    let mut command = runtime(root, &["check", "--only", "clippy"]);
    command.env("CARGO_HOME", home);
    run(command.envs(environment.iter().copied()), "")
}

fn silenced_only_outside_the_rig(root: &Path, home: &Path, environment: &[(&str, &str)]) {
    let plain = cargo(root, home, environment);
    assert_eq!(plain.code, Some(0), "{}", plain.output());
    let checked = gate(root, home, environment);
    let output = checked.output();
    assert_ne!(checked.code, Some(0), "{output}");
    let both = output.contains("unused variable") && output.contains("unwrap()");
    let blocked = output.contains("ERROR [clippy]");
    assert!(both && blocked, "{output}");
}

// DECISION: D182
#[test]
fn outside_rustflags_never_silence_the_gate() {
    let (temp, root, home) = cargo_project(FLAWED);
    silenced_only_outside_the_rig(&root, &home, &[("RUSTFLAGS", "--cap-lints allow")]);
    let parent = temp.path().canonicalize().unwrap().join(".cargo");
    for directory in [root.join(".cargo"), parent, home.clone()] {
        let config = directory.join("config.toml");
        fs::create_dir_all(&directory).unwrap();
        fs::write(&config, CAPPED).unwrap();
        silenced_only_outside_the_rig(&root, &home, &[]);
        fs::remove_file(&config).unwrap();
    }
}

// DECISION: D182
#[test]
fn the_next_cargo_run_compiles_with_the_changed_rig_list() {
    let (_temp, root, home) = cargo_project(CONDITIONAL);
    let unset = gate(&root, &home, &[]);
    assert_ne!(unset.code, Some(0), "{}", unset.output());
    assert!(
        unset.output().contains("rig_flag is unset"),
        "{}",
        unset.output()
    );
    fs::write(root.join(".agentrig/rust.yaml"), FLAGGED).unwrap();
    let set = gate(&root, &home, &[]);
    assert_eq!(set.code, Some(0), "{}", set.output());
}
