use super::evidence;
use crate::shared::testing::{REPAIR, Run, git, invoke, project};
use serde_json::Value;
use std::{fs, path::Path, path::PathBuf};
use tempfile::TempDir;

const TARGETS: &str = r#"test:
  argv: [sh, -c, 'echo "TARGETS=[$*]"', targets]
  accepts_args: true
"#;
const GROUPS: &str = "  affected:
  - {include: [src/a.rs], targets: [tests/a, tests/shared]}
  - {include: [src/b.rs, src/moved.rs], targets: [tests/b, tests/shared]}
  - {include: ['docs/**'], targets: []}
";
const EVERY_TEST: &str = "[]";

type Case = (
    &'static str,
    fn(&Path),
    fn(&Path) -> Run,
    Option<&'static str>,
);

const CASES: [Case; 8] = [
    ("edit", edit, staged, Some("[tests/a tests/shared]")),
    ("delete", delete, staged, Some("[tests/a tests/shared]")),
    (
        "rename",
        rename,
        staged,
        Some("[tests/a tests/b tests/shared]"),
    ),
    ("unknown", unknown, staged, Some(EVERY_TEST)),
    ("docs", docs, staged, None),
    ("unstaged", unstaged, staged, Some("[tests/a tests/shared]")),
    ("full", edit, full, Some(EVERY_TEST)),
    ("merge", edit, merge, Some("[tests/a tests/shared]")),
];

fn affected_project() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let check = format!("- id: tests\n  kind: command\n  command: test\n  skill: {REPAIR}\n");
    project(&root, TARGETS, &(check + GROUPS));
    fs::write(root.join(".gitignore"), ".runtime/\n").unwrap();
    fs::write(root.join("src/a.rs"), "before\n").unwrap();
    fs::write(root.join("src/b.rs"), "before\n").unwrap();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "before\n").unwrap();
    git(&root, &["init", "-q", "-b", "task/selection"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "base"]);
    (temp, root)
}

fn stage(root: &Path) {
    git(root, &["add", "-A"]);
}

fn edit(root: &Path) {
    fs::write(root.join("src/a.rs"), "changed\n").unwrap();
    stage(root);
}

fn delete(root: &Path) {
    fs::remove_file(root.join("src/a.rs")).unwrap();
    stage(root);
}

fn rename(root: &Path) {
    fs::rename(root.join("src/a.rs"), root.join("src/moved.rs")).unwrap();
    stage(root);
}

fn unknown(root: &Path) {
    fs::write(root.join("unknown.txt"), "new\n").unwrap();
    stage(root);
}

fn docs(root: &Path) {
    fs::write(root.join("docs/guide.md"), "changed\n").unwrap();
    stage(root);
}

fn unstaged(root: &Path) {
    edit(root);
    fs::write(root.join("src/b.rs"), "not staged\n").unwrap();
}

fn staged(root: &Path) -> Run {
    invoke(root, &["check", "--staged"])
}

fn full(root: &Path) -> Run {
    invoke(root, &["check"])
}

fn merge(root: &Path) -> Run {
    git(root, &["branch", "trunk"]);
    git(root, &["commit", "-qm", "feature"]);
    invoke(root, &["feature-merge"])
}

fn reported(root: &Path) -> Value {
    let report = invoke(root, &["report"]);
    assert_eq!(report.code, Some(0), "{}", report.output());
    let (_, evidence) = report.stdout.split_once("Check evidence: ").unwrap();
    serde_json::from_str(evidence.lines().next().unwrap()).unwrap()
}

#[test]
fn affected_groups_select_commit_and_merge_targets() {
    for (name, change, check, targets) in CASES {
        let (_temp, root) = affected_project();
        change(&root);
        let result = check(&root);
        assert_eq!(result.code, Some(0), "{name}: {}", result.output());
        let expected = targets.map(|targets| format!("TARGETS={targets}"));
        let ran = result.stdout.lines().find(|line| line.contains("TARGETS="));
        assert_eq!(ran, expected.as_deref(), "{name}: {}", result.stdout);
        let full = targets == Some(EVERY_TEST);
        let record = evidence(&root);
        assert_eq!(record["selective"], !full, "{name}: {record}");
        assert_eq!(reported(&root)["full_gate_passed"], full, "{name}");
    }
}
