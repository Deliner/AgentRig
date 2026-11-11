use super::{gated, stage_an_oversized_source};
use crate::shared::testing::{FAIL, GATE, REPAIR, Run, git, invoke, project, rerun, run, runtime};
use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime},
};

const NOISY: &str = "noisy:
  argv: [sh, -c, 'echo hidden; echo shown; exit 3']
  output: {failure: {tail: 1}}
";
const VALUE: &str = "value:
  argv: [cargo, run, --offline, --manifest-path, crate/Cargo.toml]
";
const MANIFEST: &str = "[package]\nname = \"value\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

fn program(value: i32) -> String {
    format!(
        "fn main() {{
    let manifest = env!(\"CARGO_MANIFEST_DIR\");
    println!(\"manifest {{manifest}}\");
    let present = std::path::Path::new(manifest).join(\"Cargo.toml\").exists();
    std::process::exit(if present {{ {value} }} else {{ 7 }});
}}
"
    )
}

fn staged_check(root: &Path, temporary: &Path) -> Run {
    let mut command = runtime(root, &["check", "--staged"]);
    run(command.env("TMPDIR", temporary), "")
}

fn rebuilt(result: &Run, code: i32) {
    assert_eq!(result.code, Some(code), "{}", result.output());
    assert!(
        result.output().contains("Compiling value"),
        "{}",
        result.output()
    );
}

// DECISION: D160
// DECISION: D161
#[test]
fn staged_cargo_check_reuses_unchanged_builds_and_checks_only_staged_code() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let first = tempfile::tempdir().unwrap();
    let check = format!("- id: value\n  kind: command\n  command: value\n  skill: {REPAIR}\n");
    project(&root, VALUE, &check);
    let main = root.join("crate/src/main.rs");
    fs::create_dir_all(main.parent().unwrap()).unwrap();
    fs::write(root.join("crate/Cargo.toml"), MANIFEST).unwrap();
    fs::write(&main, program(0)).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    rebuilt(&staged_check(&root, first.path()), 0);
    fs::write(root.join("notes.md"), "note\n").unwrap();
    git(&root, &["add", "notes.md"]);
    fs::write(&main, program(3)).unwrap();
    let reused = staged_check(&root, first.path());
    assert_eq!(reused.code, Some(0), "{}", reused.output());
    assert!(
        !reused.output().contains("Compiling"),
        "{}",
        reused.output()
    );
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
    let file = fs::File::options().write(true).open(&main).unwrap();
    file.set_modified(old).unwrap();
    git(&root, &["add", "crate/src/main.rs"]);
    let changed = staged_check(&root, first.path());
    assert_eq!(changed.code, Some(3), "{}", changed.output());
    first.close().unwrap();
    let second = tempfile::tempdir().unwrap();
    let moved = staged_check(&root, second.path());
    rebuilt(&moved, 3);
    let output = moved.output();
    let (_, printed) = output.split_once("manifest ").unwrap();
    let manifest = Path::new(printed.lines().next().unwrap());
    fs::create_dir(manifest.parent().unwrap()).unwrap();
    rebuilt(&staged_check(&root, second.path()), 3);
}

#[test]
fn staged_gate_uses_staged_configuration() {
    let (_temp, root) = gated();
    stage_an_oversized_source(&root);
    let lint = root.join("lint.yaml");
    let relaxed = fs::read_to_string(&lint)
        .unwrap()
        .replace("error: 60", "error: 100");
    fs::write(&lint, relaxed).unwrap();
    let working = invoke(&root, &["check"]);
    assert_eq!(working.code, Some(0), "{}", working.output());
    let staged = invoke(&root, &["check", "--staged"]);
    assert_eq!(staged.code, Some(1), "{}", staged.output());
    assert!(staged.stdout.contains("exceeds 60"), "{}", staged.stdout);
}

#[test]
fn staged_command_log_outlives_the_exported_index() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let check = format!("- id: noisy\n  kind: command\n  command: noisy\n  skill: {REPAIR}\n");
    project(&root, NOISY, &check);
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    let result = invoke(&root, &["check", "--staged", "--only", "noisy"]);
    assert_eq!(result.code, Some(3), "{}", result.output());
    let (_, status) = result.stdout.split_once("noisy: exit 3; log: ").unwrap();
    let log = Path::new(status.lines().next().unwrap());
    assert!(log.starts_with(root.join(".runtime")), "{}", log.display());
    let output = fs::read_to_string(log.join("stdout.log")).unwrap();
    assert_eq!(output, "hidden\nshown\n");
}

#[test]
fn staged_check_preserves_invalid_unstaged_configuration() {
    let (_temp, root) = gated();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "."]);
    let config = root.join(".agentrig/project.yaml");
    fs::write(&config, "invalid: [\n").unwrap();
    let result = invoke(&root, &["check", "--staged"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    assert_eq!(fs::read_to_string(config).unwrap(), "invalid: [\n");
}

#[test]
fn staged_diagnostic_rerun_survives_export() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp
        .path()
        .canonicalize()
        .unwrap()
        .join("project with spaces");
    project(&root, FAIL, GATE);
    stage_an_oversized_source(&root);
    let result = invoke(&root, &["check", "--staged"]);
    assert_eq!(result.code, Some(1), "{}", result.output());
    for output in [&result.stdout, &result.stderr] {
        let repeated = rerun(output);
        assert_eq!(repeated.code, Some(1), "{}", repeated.output());
        assert!(
            repeated.stdout.contains("exceeds 60"),
            "{}",
            repeated.stdout
        );
    }
}
