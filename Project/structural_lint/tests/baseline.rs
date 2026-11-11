// DECISION: D123
// DECISION: D139
// DECISION: D140
use super::consumer::{CONFIG, command, consumer, lint, write};
use crate::shared::testing::{Run, git};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

fn width() -> (TempDir, PathBuf) {
    consumer(&CONFIG.replace("nonblank-lines", "line-width"))
}

fn source(root: &Path, name: &str, long_lines: usize) {
    let text = format!("{}ok\n", "long line\n".repeat(long_lines));
    write(root, &format!("src/{name}"), &text);
}

fn record(root: &Path) -> Run {
    let recorded = command(root, "lint-baseline", &[]);
    assert_eq!(recorded.code, Some(0), "{}", recorded.output());
    recorded
}

fn baseline(root: &Path) -> Value {
    let text = fs::read_to_string(root.join("lint-baseline.yaml")).unwrap();
    serde_yaml_ng::from_str(&text).unwrap()
}

fn code(root: &Path) -> Option<i32> {
    lint(root).0
}

fn levels(items: &[Value]) -> Vec<&str> {
    items
        .iter()
        .map(|item| item["level"].as_str().unwrap())
        .collect()
}

fn message(item: &Value) -> &str {
    item["message"].as_str().unwrap()
}

fn commit_all(root: &Path) {
    git(root, &["init", "-q"]);
    git(root, &["add", "."]);
    let identity = [
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
    ];
    git(root, &[&identity[..], &["commit", "-qm", "state"]].concat());
}

#[test]
fn baseline_accepts_recorded_errors_and_fails_when_a_path_gains_one() {
    let (_temp, root) = width();
    source(&root, "a.rs", 2);
    assert_eq!(code(&root), Some(1));
    record(&root);
    assert_eq!(baseline(&root), json!({"source": {"src/a.rs": 2}}));
    let (accepted, items) = lint(&root);
    assert_eq!(accepted, Some(0));
    assert_eq!(levels(&items), ["warning", "warning"]);
    let marked = items
        .iter()
        .all(|item| message(item).ends_with("accepted by the lint baseline"));
    assert!(marked, "{items:?}");
    source(&root, "a.rs", 3);
    let (exceeded, items) = lint(&root);
    assert_eq!(exceeded, Some(1));
    assert_eq!(levels(&items), ["error"; 3]);
    assert!(message(&items[0]).contains("accepts 2 of these 3 findings"));
}

#[test]
fn baseline_suggests_lowering_and_recording_never_raises_a_count() {
    let (_temp, root) = width();
    source(&root, "a.rs", 2);
    source(&root, "b.rs", 1);
    record(&root);
    source(&root, "a.rs", 1);
    let lowered = command(&root, "lint", &[]);
    assert_eq!(lowered.code, Some(0), "{}", lowered.output());
    let suggested = "fell below the baseline for 1 rule and path pairs";
    assert!(lowered.stderr.contains(suggested), "{}", lowered.stderr);
    source(&root, "b.rs", 2);
    source(&root, "c.rs", 1);
    assert_eq!(code(&root), Some(1));
    let recorded = record(&root);
    let kept = "2 rule and path pairs exceed the baseline and were not raised";
    assert!(recorded.stdout.contains(kept), "{}", recorded.stdout);
    let expected = json!({"source": {"src/a.rs": 1, "src/b.rs": 1}});
    assert_eq!(baseline(&root), expected);
    assert_eq!(code(&root), Some(1));
}

#[test]
fn recording_restores_a_lowered_count_up_to_the_committed_baseline() {
    let (_temp, root) = width();
    source(&root, "a.rs", 2);
    source(&root, "b.rs", 1);
    record(&root);
    commit_all(&root);
    source(&root, "a.rs", 0);
    record(&root);
    assert_eq!(baseline(&root), json!({"source": {"src/b.rs": 1}}));
    source(&root, "a.rs", 2);
    assert_eq!(code(&root), Some(1));
    record(&root);
    let restored = json!({"source": {"src/a.rs": 2, "src/b.rs": 1}});
    assert_eq!(baseline(&root), restored);
    assert_eq!(code(&root), Some(0));
    source(&root, "a.rs", 3);
    source(&root, "c.rs", 1);
    record(&root);
    assert_eq!(baseline(&root), restored);
    assert_eq!(code(&root), Some(1));
}

#[test]
fn recording_keeps_a_rerecorded_baseline_and_lowers_an_uncommitted_one() {
    let (_temp, root) = width();
    source(&root, "a.rs", 2);
    commit_all(&root);
    record(&root);
    source(&root, "a.rs", 1);
    record(&root);
    source(&root, "a.rs", 2);
    record(&root);
    assert_eq!(baseline(&root), json!({"source": {"src/a.rs": 1}}));
    commit_all(&root);
    fs::remove_file(root.join("lint-baseline.yaml")).unwrap();
    source(&root, "c.rs", 1);
    record(&root);
    record(&root);
    let rerecorded = json!({"source": {"src/a.rs": 2, "src/c.rs": 1}});
    assert_eq!(baseline(&root), rerecorded);
}

#[test]
fn recording_follows_a_git_rename_without_accepting_a_new_finding() {
    let (_temp, root) = width();
    source(&root, "a.rs", 2);
    record(&root);
    commit_all(&root);
    git(&root, &["mv", "src/a.rs", "src/b.rs"]);
    assert_eq!(code(&root), Some(1));
    record(&root);
    let renamed = json!({"source": {"src/b.rs": 2}});
    assert_eq!(baseline(&root), renamed);
    assert_eq!(code(&root), Some(0));
    source(&root, "b.rs", 3);
    let recorded = record(&root);
    let kept = "1 rule and path pairs exceed the baseline and were not raised";
    assert!(recorded.stdout.contains(kept), "{}", recorded.stdout);
    assert_eq!(baseline(&root), renamed);
    assert_eq!(code(&root), Some(1));
}
