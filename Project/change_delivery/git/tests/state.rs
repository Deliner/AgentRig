// DECISION: D191
use super::{STATE, commit, installed, merge, read, start};
use crate::project_memory::format::state_template;
use crate::shared::testing::{git, invoke};
use std::fs;

#[test]
fn state_resets_after_each_feature_commit_and_reaches_the_base_empty() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "product.txt", "product");
    let template = state_template();
    assert_eq!(read(&root, STATE), template);
    let staged = git(&root, &["show", &format!(":{STATE}")]);
    assert_eq!(staged, template.trim_end());
    let committed = git(&root, &["show", &format!("HEAD:{STATE}")]);
    assert_ne!(committed, template.trim_end());
    merge(&root);
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
    assert_eq!(read(&root, STATE), template);
    let subject = git(&root, &["log", "-1", "--format=%s", "task/product"]);
    assert_eq!(subject, "Reset State before integration");
    let changed = ["diff", "--name-only", "task/product~1", "task/product"];
    assert_eq!(git(&root, &changed), STATE);
}

#[test]
fn integration_keeps_uncommitted_state_notes() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "product.txt", "product");
    let feature = git(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join(STATE), "# State\n\nNotes of the next VAC.\n").unwrap();
    let refused = invoke(&root, &["feature-merge"]);
    assert_eq!(refused.code, Some(2), "{}", refused.output());
    let named = refused.stderr.contains("uncommitted notes");
    assert!(named, "{}", refused.stderr);
    assert_eq!(read(&root, STATE), "# State\n\nNotes of the next VAC.\n");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), feature);
    assert_eq!(git(&root, &["branch", "--show-current"]), "task/product");
}
