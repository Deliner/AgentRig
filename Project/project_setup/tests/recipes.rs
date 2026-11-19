// DECISION: D201
use crate::shared::testing::{Run, configure, hermetic, invoke, run};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const ECHOING_READ: &str = "read:\n  purpose: Print words\n  argv: []\n  accepts_args: true\n  read_only: true\n  programs:\n    echo: {}\n";

fn initialized() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let result = invoke(&root, &["init"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    (temp, root)
}

fn just(root: &Path, args: &[&str]) -> Run {
    run(hermetic("just").args(args).current_dir(root), "")
}

#[test]
fn recipes_run_on_the_project_of_their_justfile_whatever_root_the_words_name() {
    let (_project_temp, project) = initialized();
    let (_other_temp, other) = initialized();
    configure(&other, "commands", ECHOING_READ);
    let echoed = just(&other, &["read", "echo", "moved"]);
    assert_eq!(echoed.stdout, "moved\n", "{}", echoed.output());
    let other_root = other.to_str().unwrap();
    let moved = just(&project, &["read", "echo", "--root", other_root, "moved"]);
    assert!(!moved.stdout.contains("moved"), "{}", moved.output());
    let own_catalog = moved.output().contains(" does not run ");
    assert!(own_catalog, "{}", moved.output());
}
