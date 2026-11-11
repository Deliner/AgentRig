use super::{installed, read};
use crate::shared::testing::{commit, git, invoke};
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

#[test]
fn feature_start_rejects_pending_changes_and_invalid_names() {
    let cases = [
        (Some("shared.txt"), "product"),
        (Some("pending.txt"), "product"),
        (None, "bad:name"),
    ];
    for (pending, name) in cases {
        let (_temp, root) = installed();
        let base = git(&root, &["rev-parse", "HEAD"]);
        if let Some(file) = pending {
            fs::write(root.join(file), "pending\n").unwrap();
        }
        let refused = invoke(&root, &["feature-start", name]);
        assert_eq!(
            refused.code,
            Some(2),
            "{pending:?} {name}: {}",
            refused.output()
        );
        assert_eq!(git(&root, &["rev-parse", "HEAD"]), base);
        assert_eq!(git(&root, &["branch", "--show-current"]), "trunk");
        if let Some(file) = pending {
            assert_eq!(read(&root, file), "pending\n");
        }
    }
}
