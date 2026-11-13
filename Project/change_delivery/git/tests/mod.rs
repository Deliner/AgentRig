mod cleanup;
mod guards;
mod integration;
mod push;
mod start;

use crate::shared::testing::{configure, git, invoke};
use std::{fs, path::Path, path::PathBuf};
use tempfile::TempDir;

const PROBE_COMMAND: &str = r#"probe:
  purpose: Block delivery on request
  argv: [sh, -c, 'test -z "$BLOCK_DELIVERY"']
"#;
const PROBE_CHECK: &str = "- id: probe
  kind: command
  command: probe
  skill: .agentrig/skills/repair/SKILL.md
";

fn installed() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    fs::write(root.join("shared.txt"), "initial\n").unwrap();
    let initialized = invoke(&root, &["init", "--base", "trunk", "--prefix", "task/"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    configure(&root, "commands", PROBE_COMMAND);
    configure(&root, "checks", PROBE_CHECK);
    git(&root, &["add", "."]);
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    git(
        &root,
        &[&unhooked[..], &["commit", "-qm", "installed"]].concat(),
    );
    (temp, root)
}

fn start(root: &Path, name: &str) {
    let started = invoke(root, &["feature-start", name]);
    assert_eq!(started.code, Some(0), "{}", started.output());
}

fn merge(root: &Path) {
    let merged = invoke(root, &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
}

fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}
