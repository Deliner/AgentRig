use super::{commit, installed, read, start};
use crate::shared::testing::{git, invoke};
use std::fs;

#[test]
fn feature_start_preserves_the_parent_and_rejects_an_existing_branch() {
    let (_temp, root) = installed();
    let base = git(&root, &["rev-parse", "HEAD"]);
    let started = invoke(&root, &["feature-start", "product"]);
    assert_eq!(started.code, Some(0), "{}", started.output());
    assert_eq!(started.stdout.trim(), "task/product");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    assert_eq!(git(&root, &["branch", "--show-current"]), "task/product");
    let nested = invoke(&root, &["feature-start", "another"]);
    assert_eq!(nested.code, Some(2), "{}", nested.output());
    commit(&root, "shared.txt", "changed\n");
    git(&root, &["switch", "trunk"]);
    let existing = invoke(&root, &["feature-start", "product"]);
    assert_eq!(existing.code, Some(2), "{}", existing.output());
    assert!(existing.stderr.contains("exists"), "{}", existing.stderr);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    assert_eq!(read(&root, "shared.txt"), "initial\n");
}

// DECISION: D194
#[test]
fn feature_start_carries_pending_changes_to_the_new_branch() {
    let (_temp, root) = installed();
    let base = git(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join("shared.txt"), "pending\n").unwrap();
    fs::write(root.join("pending.txt"), "pending\n").unwrap();
    let started = invoke(&root, &["feature-start", "product"]);
    assert_eq!(started.code, Some(0), "{}", started.output());
    assert_eq!(git(&root, &["branch", "--show-current"]), "task/product");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    assert_eq!(read(&root, "shared.txt"), "pending\n");
    assert_eq!(read(&root, "pending.txt"), "pending\n");
}

#[test]
fn feature_start_rejects_a_pending_merge_and_an_invalid_name() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "shared.txt", "feature\n");
    git(&root, &["switch", "trunk"]);
    let base = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["merge", "--no-ff", "--no-commit", "task/product"]);
    let merging = invoke(&root, &["feature-start", "other"]);
    assert_eq!(merging.code, Some(2), "{}", merging.output());
    let named = merging.stderr.contains("pending VCS operation");
    assert!(named, "{}", merging.stderr);
    git(&root, &["merge", "--abort"]);
    let invalid = invoke(&root, &["feature-start", "bad:name"]);
    assert_eq!(invalid.code, Some(2), "{}", invalid.output());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
    assert_eq!(git(&root, &["branch", "--show-current"]), "trunk");
}
