// DECISION: D003
// DECISION: D011
// DECISION: D014
use super::{installed, read, start};
use crate::shared::testing::{git, hermetic, run, runtime};
use std::fs;

// INVARIANT: I006
#[test]
fn staged_commit_preserves_unstaged_work() {
    let (_temp, root) = installed();
    start(&root, "atomic");
    let before = git(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join("product.txt"), "staged").unwrap();
    git(&root, &["add", "product.txt"]);
    fs::write(root.join("product.txt"), "later unstaged work").unwrap();
    fs::write(root.join("unrelated.txt"), "user work").unwrap();
    let mut blocked = hermetic("git");
    blocked
        .args(["commit", "-qm", "blocked"])
        .current_dir(&root)
        .env("BLOCK_DELIVERY", "1");
    let blocked = run(&mut blocked, "");
    assert_ne!(blocked.code, Some(0), "{}", blocked.output());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), before);
    git(&root, &["commit", "-qm", "verified index"]);
    assert_eq!(git(&root, &["show", "HEAD:product.txt"]), "staged");
    assert_eq!(read(&root, "product.txt"), "later unstaged work");
    let committed = git(&root, &["ls-tree", "--name-only", "HEAD"]);
    assert!(!committed.lines().any(|name| name == "unrelated.txt"));
    assert_eq!(read(&root, "unrelated.txt"), "user work");
}

// INVARIANT: I021
#[test]
fn reference_guard_rejects_deletion_when_the_tip_cannot_be_resolved() {
    let (_temp, root) = installed();
    git(
        &root,
        &["-c", "core.hooksPath=/dev/null", "branch", "task/merged"],
    );
    let unknown = "0".repeat(40);
    let delete = |branch: &str| {
        let update = format!("{unknown} {unknown} refs/heads/task/{branch}\n");
        run(
            &mut runtime(&root, &["guard-reference", "prepared"]),
            &update,
        )
    };
    let merged = delete("merged");
    assert_eq!(merged.code, Some(1), "{}", merged.output());
    assert!(merged.stderr.contains("retained"), "{}", merged.stderr);
    let missing = delete("missing");
    assert_eq!(missing.code, Some(2), "{}", missing.output());
    assert!(!missing.stderr.contains("retained"), "{}", missing.stderr);
}
