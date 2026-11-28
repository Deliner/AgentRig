mod approval;
mod cleanup;
mod compaction;
mod guards;
mod integration;
mod pin;
mod push;
mod start;
mod state;
mod sweep;

use crate::shared::testing::{
    Run, commit_identity, configure, describe_state, git, git_run, invoke,
};
use std::{fs, path::Path, path::PathBuf};
use tempfile::TempDir;

const STATE: &str = "Ledger/State.md";
const DECISION: &str =
    "# D001\n\n## Decision\n\nText.\n\n## Reasons\n\n- Text.\n\n## Alternatives\n\n- Text.\n";

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
    commit_identity(&root);
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

fn approve_commands(root: &Path, addition: &str) {
    configure(root, "commands", addition);
    let approved = invoke(root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
}

fn merge(root: &Path) {
    let merged = invoke(root, &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
}

fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}

fn committed(root: &Path, message: &str) -> Run {
    describe_state(root, "A test VAC.");
    git(root, &["add", "."]);
    git_run(root, &["commit", "-qm", message])
}

fn commit(root: &Path, name: &str, text: &str) {
    fs::write(root.join(name), text).unwrap();
    describe_state(root, "A test VAC.");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", name]);
}
