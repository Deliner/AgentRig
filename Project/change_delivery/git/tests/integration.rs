// DECISION: D010
// DECISION: D012
use super::{installed, merge, read, start};
use crate::shared::testing::{commit, configure, git, git_run, invoke, run, runtime};
use std::{fs, path::Path};

const MUTATING_COMMAND: &str = r#"probe:
  purpose: Mutate a gate input on request
  argv: [sh, -c, 'test -z "$MUTATE_MERGE" || echo changed > product.txt']
"#;

fn diverged(root: &Path) -> String {
    start(root, "product");
    commit(root, "product.txt", "product");
    git(root, &["switch", "-c", "task/product-side"]);
    commit(root, "side.txt", "side");
    git(root, &["switch", "task/product"]);
    commit(root, "main.txt", "main");
    git(
        root,
        &["merge", "--no-ff", "--no-edit", "task/product-side"],
    );
    git(root, &["switch", "trunk"]);
    start(root, "concurrent");
    commit(root, "concurrent.txt", "concurrent");
    merge(root);
    git(root, &["rev-parse", "trunk"])
}

// INVARIANT: I010
#[test]
fn divergent_rebase_preserves_merge_history() {
    let (_temp, root) = installed();
    let initial = git(&root, &["rev-parse", "HEAD"]);
    let direct = git_run(
        &root,
        &["commit", "--allow-empty", "-qm", "direct base commit"],
    );
    assert_ne!(direct.code, Some(0), "{}", direct.output());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), initial);
    let base = diverged(&root);
    git(&root, &["switch", "task/product"]);
    merge(&root);
    assert_eq!(git(&root, &["branch", "--show-current"]), "trunk");
    let tip = git(&root, &["rev-parse", "task/product"]);
    let rebased = git(&root, &["rev-list", "--parents", "-1", &tip]);
    assert_eq!(rebased.split_whitespace().count(), 3, "{rebased}");
    let integrated = git(&root, &["rev-list", "--parents", "-1", "trunk"]);
    let parents: Vec<_> = integrated.split_whitespace().skip(1).collect();
    assert_eq!(parents, [base.as_str(), tip.as_str()]);
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
    for name in ["product", "side", "main", "concurrent"] {
        assert_eq!(read(&root, &format!("{name}.txt")), name);
    }
    let deleted = git_run(&root, &["branch", "-D", "task/product"]);
    assert_ne!(deleted.code, Some(0), "{}", deleted.output());
    assert_eq!(git(&root, &["rev-parse", "task/product"]), tip);
}

#[test]
fn conflicting_rebase_can_be_aborted_without_losing_the_feature() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "shared.txt", "product\n");
    let product = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["switch", "trunk"]);
    start(&root, "concurrent");
    commit(&root, "shared.txt", "concurrent\n");
    merge(&root);
    let base = git(&root, &["rev-parse", "trunk"]);
    git(&root, &["switch", "task/product"]);
    let conflicted = invoke(&root, &["feature-merge"]);
    assert_ne!(conflicted.code, Some(0), "{}", conflicted.output());
    assert!(
        conflicted.output().contains("CONFLICT"),
        "{}",
        conflicted.output()
    );
    assert_eq!(git(&root, &["rev-parse", "trunk"]), base);
    assert_ne!(git(&root, &["ls-files", "--unmerged"]), "");
    git(&root, &["rebase", "--abort"]);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), product);
    assert_eq!(read(&root, "shared.txt"), "product\n");
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
}

#[test]
fn gate_failure_and_dirty_workspace_preserve_branches() {
    let (_temp, root) = installed();
    start(&root, "product");
    commit(&root, "product.txt", "product");
    let feature = git(&root, &["rev-parse", "HEAD"]);
    let base = git(&root, &["rev-parse", "trunk"]);
    let blocked = run(
        runtime(&root, &["feature-merge"]).env("BLOCK_DELIVERY", "1"),
        "",
    );
    assert_eq!(blocked.code, Some(1), "{}", blocked.output());
    assert!(
        blocked.stderr.contains("ERROR [probe]"),
        "{}",
        blocked.stderr
    );
    assert!(
        blocked.stderr.contains("repair/SKILL.md"),
        "{}",
        blocked.stderr
    );
    assert_eq!(git(&root, &["branch", "--show-current"]), "task/product");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), feature);
    assert_eq!(git(&root, &["rev-parse", "trunk"]), base);
    fs::write(root.join("product.txt"), "uncommitted").unwrap();
    let dirty = invoke(&root, &["feature-merge"]);
    assert_eq!(dirty.code, Some(2), "{}", dirty.output());
    assert!(
        dirty.stderr.contains("working tree must be clean"),
        "{}",
        dirty.stderr
    );
    assert_eq!(read(&root, "product.txt"), "uncommitted");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), feature);
}

#[test]
fn integration_rejects_mutated_gate_inputs() {
    let (_temp, root) = installed();
    start(&root, "mutating-check");
    configure(&root, "commands", MUTATING_COMMAND);
    commit(&root, "product.txt", "verified");
    let feature = git(&root, &["rev-parse", "HEAD"]);
    let base = git(&root, &["rev-parse", "trunk"]);
    let mutated = run(
        runtime(&root, &["feature-merge"]).env("MUTATE_MERGE", "1"),
        "",
    );
    assert_eq!(mutated.code, Some(2), "{}", mutated.output());
    assert!(
        mutated.stderr.contains("checked merge inputs changed"),
        "{}",
        mutated.stderr
    );
    assert_eq!(
        git(&root, &["branch", "--show-current"]),
        "task/mutating-check"
    );
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), feature);
    assert_eq!(git(&root, &["rev-parse", "trunk"]), base);
    assert_eq!(read(&root, "product.txt"), "changed\n");
}

#[test]
fn refused_switch_to_base_keeps_the_feature_tree() {
    let (_temp, root) = installed();
    let hook = ".agentrig/hooks/reference-transaction";
    let generated = read(&root, hook);
    fs::write(root.join(hook), "#!/bin/sh\nexit 1\n").unwrap();
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    let refusing = ["commit", "-qam", "base hook refuses every reference update"];
    git(&root, &[&unhooked[..], &refusing].concat());
    git(
        &root,
        &[&unhooked[..], &["switch", "-c", "task/product"]].concat(),
    );
    commit(&root, hook, &generated);
    let feature = git(&root, &["rev-parse", "HEAD"]);
    let refused = invoke(&root, &["feature-merge"]);
    assert_eq!(refused.code, Some(2), "{}", refused.output());
    assert!(
        refused.stderr.contains("aborted by hook"),
        "{}",
        refused.stderr
    );
    assert_eq!(git(&root, &["branch", "--show-current"]), "task/product");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), feature);
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
    assert_eq!(read(&root, hook), generated);
}
